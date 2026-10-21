//! Opt-in terminal frontend. Demo widgets have no module/service access.
mod portrait;
mod render;
mod terminal;
mod theme;
use crate::session::{self, Action, Session};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use heart_core::{Engine, Inference, Personality};
pub use portrait::{PASTEL_PNG, contain_pixels};
pub use render::draw;
use std::{
    io,
    sync::mpsc::{self, Receiver, SyncSender, TryRecvError},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Shell,
    Input,
    Dialogue,
    Music,
}
struct Work {
    request: String,
    profile: Personality,
}
enum Update {
    Progress(String, String),
    Done(String, Vec<String>),
}
struct Worker {
    requests: SyncSender<Work>,
    updates: Receiver<Update>,
}
impl Worker {
    fn spawn(
        mut task: impl FnMut(Work, &mpsc::Sender<Update>) + Send + 'static,
    ) -> io::Result<Self> {
        let (requests, receive) = mpsc::sync_channel(1);
        let (send, updates) = mpsc::channel();
        thread::Builder::new()
            .name("heart-command".into())
            .spawn(move || {
                while let Ok(work) = receive.recv() {
                    task(work, &send);
                }
            })?;
        Ok(Self { requests, updates })
    }
}
/// UI-only state; constructing a preview never initializes an engine or reads configuration.
pub struct Dashboard {
    pub session: Session,
    pub input: String,
    cursor: usize, // UTF-8 byte boundary
    pub focus: Focus,
    pub help: bool,
    help_scroll: u16,
    pub busy: bool,
    pub dialogue: Vec<String>,
    pub scroll: u16,
    pub playing: bool,
    pub track: usize,
    pub tick: u64,
    pub animations: bool,
    history: Vec<String>,
    history_index: Option<usize>,
    draft: String,
    portrait: portrait::Portrait,
    /// Last production portrait bounds; absent when hidden by help or compact layout.
    pub portrait_rect: Option<ratatui::layout::Rect>,
    shell: terminal::TerminalPane,
    companion: String,
}
fn speaker(profile: &Personality) -> &str {
    profile
        .appearance
        .as_ref()
        .map(|a| a.display_name.as_str())
        .unwrap_or_else(|| profile.name())
}
impl Dashboard {
    pub fn new(session: Session, animations: bool) -> Self {
        let greeting = format!(
            "{} · {}",
            speaker(session.profile()),
            session.profile().greeting()
        );
        Self {
            session,
            input: String::new(),
            cursor: 0,
            focus: Focus::Shell,
            help: false,
            help_scroll: 0,
            busy: false,
            dialogue: vec![greeting.clone()],
            companion: greeting,
            scroll: u16::MAX,
            playing: false,
            track: 0,
            tick: 0,
            animations,
            history: vec![],
            history_index: None,
            draft: String::new(),
            portrait: portrait::Portrait::default(),
            portrait_rect: None,
            shell: terminal::TerminalPane::preview(),
        }
    }
    /// Deterministic screen fixture; never starts a child or exposes an input handle.
    pub fn preview_terminal(&mut self, width: u16, height: u16, output: &[u8]) {
        let area = render::areas(ratatui::layout::Rect::new(0, 0, width, height));
        self.shell
            .resize(area.shell_inner.height, area.shell_inner.width);
        self.shell.preview_output(output);
    }
    fn append(&mut self, line: String) {
        // Bound session memory and paragraph scroll offsets; no persistent conversation.
        self.companion = line.clone();
        self.dialogue.push(line);
        if self.dialogue.len() > 80 {
            self.dialogue.remove(0);
        }
        self.scroll = u16::MAX;
    }
    fn normalize_focus(&mut self, width: u16, height: u16) {
        let areas = render::areas(ratatui::layout::Rect::new(0, 0, width, height));
        if (self.focus == Focus::Music && areas.music.is_empty())
            || (self.focus == Focus::Dialogue
                && (areas.dialogue.width < 3 || areas.dialogue.height < 3))
        {
            self.focus = Focus::Input;
        }
    }

    fn switch_profile(&mut self) {
        self.session.cycle();
        self.append(format!(
            "{} · {}",
            speaker(self.session.profile()),
            self.session.profile().greeting()
        ));
    }
    fn submit(&mut self, worker: &Worker) -> bool {
        // Local commands stay available while working; requests retain the editable draft.
        let request = self.input.clone();
        let action = self.session.classify(&request);
        if matches!(action, Action::Request(_)) && self.busy {
            return false;
        }
        if !request.trim().is_empty() {
            if self.history.last() != Some(&request) {
                self.history.push(request.clone());
            }
            if self.history.len() > 50 {
                self.history.remove(0);
            }
        }
        self.input.clear();
        self.cursor = 0;
        self.history_index = None;
        self.draft.clear();
        match action {
            Action::Exit => return true,
            Action::Empty => {}
            Action::Help => self.help = true,
            Action::Profile(text) => {
                self.append(format!("{} · {text}", speaker(self.session.profile())))
            }
            Action::Message(text) => self.append(text),
            Action::Request(request) => {
                let profile = self.session.profile().clone();
                self.append(format!("> {request}"));
                match worker.requests.try_send(Work { request, profile }) {
                    Ok(()) => self.busy = true,
                    Err(_) => self.append("Command worker unavailable; restart HEART.".into()),
                }
            }
        }
        false
    }
    fn updates(&mut self, worker: &Worker) {
        loop {
            match worker.updates.try_recv() {
                Ok(Update::Progress(name, text)) => self.append(format!("{name} · {text}")),
                Ok(Update::Done(name, lines)) => {
                    self.busy = false;
                    let companion = format!("{name} · {}", lines.join("\n"));
                    for text in lines {
                        self.append(format!("{name} · {text}"));
                    }
                    self.companion = companion;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    if self.busy {
                        self.busy = false;
                        self.append("Command worker stopped; restart HEART.".into());
                    }
                    break;
                }
            }
        }
    }
    fn key(&mut self, key: KeyEvent, worker: &Worker, width: u16, height: u16) -> bool {
        if key.kind == KeyEventKind::Release {
            return false;
        }
        if key.code == KeyCode::F(12) {
            self.help = false;
            self.focus = if self.focus == Focus::Shell {
                Focus::Input
            } else {
                Focus::Shell
            };
            return false;
        }
        if self.focus == Focus::Shell {
            self.shell.key(key);
            return false;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('q' | 'c'))
        {
            return true;
        }
        match key.code {
            KeyCode::F(1) => {
                self.help = !self.help;
                return false;
            }
            KeyCode::Esc => {
                self.help = false;
                self.focus = Focus::Input;
                return false;
            }
            KeyCode::F(2) => {
                self.switch_profile();
                return false;
            }
            _ if self.help => {
                match key.code {
                    KeyCode::Up => self.help_scroll = self.help_scroll.saturating_sub(1),
                    KeyCode::Down => self.help_scroll = self.help_scroll.saturating_add(1),
                    KeyCode::PageUp => self.help_scroll = self.help_scroll.saturating_sub(5),
                    KeyCode::PageDown => self.help_scroll = self.help_scroll.saturating_add(5),
                    KeyCode::Home => self.help_scroll = 0,
                    KeyCode::End => self.help_scroll = u16::MAX,
                    _ => {}
                }
                return false;
            }
            KeyCode::Tab | KeyCode::BackTab => {
                let mut choices = vec![Focus::Shell, Focus::Input];
                let areas = render::areas(ratatui::layout::Rect::new(0, 0, width, height));
                if areas.dialogue.width >= 3 && areas.dialogue.height >= 3 {
                    choices.push(Focus::Dialogue);
                }
                if !areas.music.is_empty() {
                    choices.push(Focus::Music);
                }
                let pos = choices.iter().position(|f| *f == self.focus).unwrap_or(0);
                let backwards =
                    key.code == KeyCode::BackTab || key.modifiers.contains(KeyModifiers::SHIFT);
                self.focus =
                    choices[(pos + if backwards { choices.len() - 1 } else { 1 }) % choices.len()];
            }
            _ if self.focus == Focus::Dialogue => match key.code {
                KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
                KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
                KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(5),
                KeyCode::PageDown => self.scroll = self.scroll.saturating_add(5),
                KeyCode::Home => self.scroll = 0,
                KeyCode::End => self.scroll = u16::MAX,
                _ => {}
            },
            _ if self.focus == Focus::Music => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => self.playing = !self.playing,
                KeyCode::Left => self.track = (self.track + 2) % 3,
                KeyCode::Right => self.track = (self.track + 1) % 3,
                _ => {}
            },
            KeyCode::Enter => return self.submit(worker),
            KeyCode::Left => {
                self.cursor = self.input[..self.cursor]
                    .char_indices()
                    .last()
                    .map(|(i, _)| i)
                    .unwrap_or(0)
            }
            KeyCode::Right => {
                self.cursor += self.input[self.cursor..]
                    .chars()
                    .next()
                    .map(char::len_utf8)
                    .unwrap_or(0)
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.input.len(),
            KeyCode::Backspace => {
                if let Some((start, _)) = self.input[..self.cursor].char_indices().last() {
                    self.input.drain(start..self.cursor);
                    self.cursor = start;
                }
            }
            KeyCode::Delete => {
                if let Some(c) = self.input[self.cursor..].chars().next() {
                    self.input.drain(self.cursor..self.cursor + c.len_utf8());
                }
            }
            KeyCode::Up | KeyCode::Down => {
                if !self.history.is_empty() {
                    if self.history_index.is_none() {
                        self.draft = self.input.clone();
                    }
                    let index = self.history_index.unwrap_or(self.history.len());
                    let next = if key.code == KeyCode::Up {
                        index.saturating_sub(1)
                    } else {
                        (index + 1).min(self.history.len())
                    };
                    self.history_index = if next == self.history.len() {
                        None
                    } else {
                        Some(next)
                    };
                    self.input = self
                        .history_index
                        .map(|i| self.history[i].clone())
                        .unwrap_or_else(|| self.draft.clone());
                    self.cursor = self.input.len();
                }
            }
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                    && !c.is_control()
                    && self.input.chars().count() < 2000 =>
            {
                self.input.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
            _ => {}
        }
        false
    }
    fn paste(&mut self, text: &str) {
        if self.focus == Focus::Shell {
            self.shell.paste(text);
        } else if self.focus == Focus::Input && !self.help {
            for c in text
                .chars()
                .filter(|c| !c.is_control())
                .take(2000usize.saturating_sub(self.input.chars().count()))
            {
                self.input.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
        }
    }
}
struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::execute!(io::stdout(), event::DisableBracketedPaste);
        ratatui::restore();
    }
}
pub fn run<I: Inference + Send + 'static>(
    mut engine: Engine<I>,
    session: Session,
    animations: bool,
) -> io::Result<()> {
    let diagnostics = session.diagnostics;
    let worker = Worker::spawn(move |work, updates| {
        let name = speaker(&work.profile).to_owned();
        let result = session::execute(&mut engine, &work.request, &work.profile, |text| {
            let _ = updates.send(Update::Progress(name.clone(), text.to_owned()));
        });
        let lines = match result {
            Ok(reply) => session::reply_lines(reply, diagnostics),
            Err(error) => vec![error],
        };
        let _ = updates.send(Update::Done(name, lines));
    })?;
    // Ratatui installs its restoring panic hook before entering raw/alternate mode.
    // Guard is created first so even partially failed initialization is restored.
    let _guard = TerminalGuard;
    let mut terminal = ratatui::try_init()?;
    crossterm::execute!(io::stdout(), event::EnableBracketedPaste)?;
    let mut app = Dashboard::new(session, animations);
    app.portrait = portrait::Portrait::interactive();
    let size = terminal.size()?;
    let areas = render::areas(ratatui::layout::Rect::new(0, 0, size.width, size.height));
    app.shell = terminal::TerminalPane::spawn(areas.shell_inner.height, areas.shell_inner.width);
    if app.shell.ended() {
        app.focus = Focus::Input;
    }
    let mut last_tick = Instant::now();
    loop {
        app.updates(&worker);
        let size = terminal.size()?;
        let areas = render::areas(ratatui::layout::Rect::new(0, 0, size.width, size.height));
        app.shell
            .resize(areas.shell_inner.height, areas.shell_inner.width);
        let was_ended = app.shell.ended();
        app.shell.drain();
        if !was_ended && app.shell.ended() && app.focus == Focus::Shell {
            app.focus = Focus::Input;
        }
        app.normalize_focus(size.width, size.height);
        terminal.draw(|frame| draw(frame, &mut app))?;
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if app.key(key, &worker, size.width, size.height) => break,
                Event::Paste(text) => app.paste(&text),
                Event::Resize(..) => app.portrait.refresh_metrics(),
                _ => {}
            }
        }
        if last_tick.elapsed() >= Duration::from_millis(500) {
            if app.animations {
                app.tick = app.tick.wrapping_add(1);
            }
            last_tick = Instant::now();
        }
    }
    // Dropping channels detaches the one bounded worker; never join a network deadline on quit.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};
    use std::collections::BTreeMap;
    fn app() -> Dashboard {
        let mut app = Dashboard::new(
            Session {
                profiles: BTreeMap::from([
                    ("cat".into(), Personality::cat()),
                    ("butler".into(), Personality::butler()),
                ]),
                selected: "cat".into(),
                help: "/disk [path]\n/uptime\n/profile ID\n/help\n/exit".into(),
                experimental_nl: false,
                diagnostics: false,
            },
            false,
        );
        app.focus = Focus::Input;
        app
    }
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    fn backend(app: &mut Dashboard, w: u16, h: u16) -> TestBackend {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|f| draw(f, app)).unwrap();
        terminal.backend().clone()
    }
    fn plain(backend: &TestBackend) -> String {
        backend
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }
    #[test]
    fn shell_focus_owns_chords_and_paste_f12_restores_contextual_heart_controls() {
        let (sent, requests) = mpsc::channel();
        let worker = Worker::spawn(move |work, _| {
            sent.send(work.request).unwrap();
        })
        .unwrap();
        let mut app = Dashboard::new(app().session, false);
        assert_eq!(app.focus, Focus::Shell);
        for key in [
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
            key(KeyCode::Tab),
            key(KeyCode::F(1)),
            key(KeyCode::F(2)),
            key(KeyCode::Enter),
        ] {
            assert!(!app.key(key, &worker, 120, 40));
            assert_eq!(app.focus, Focus::Shell);
            assert_eq!(app.session.selected, "cat");
            assert!(!app.help);
        }
        app.paste("/uptime\n");
        assert!(app.input.is_empty());
        assert!(requests.try_recv().is_err());
        app.key(key(KeyCode::F(12)), &worker, 120, 40);
        assert_eq!(app.focus, Focus::Input);
        app.paste("/uptime\n");
        assert_eq!(app.input, "/uptime");
        assert!(requests.try_recv().is_err());
        app.key(key(KeyCode::Enter), &worker, 120, 40);
        assert_eq!(
            requests.recv_timeout(Duration::from_secs(1)).unwrap(),
            "/uptime"
        );
        app.key(key(KeyCode::F(1)), &worker, 120, 40);
        assert!(app.help);
        app.key(key(KeyCode::F(12)), &worker, 120, 40);
        assert!(!app.help);
        assert_eq!(app.focus, Focus::Shell);
        app.key(key(KeyCode::F(12)), &worker, 120, 40);
        app.key(key(KeyCode::F(2)), &worker, 120, 40);
        assert_eq!(app.session.selected, "butler");
        assert!(app.key(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &worker,
            120,
            40
        ));
    }
    #[test]
    fn compact_companion_starts_with_answer_and_history_keeps_sources_and_diagnostics() {
        let mut app = app();
        let (requests, _) = mpsc::sync_channel(1);
        let (send, updates) = mpsc::channel();
        let worker = Worker { requests, updates };
        send.send(Update::Done(
            "Pastel".into(),
            vec![
                "The answer stays visible.".into(),
                "Source [1]: https://example.com/".repeat(20),
                "Diagnostics: authored".into(),
            ],
        ))
        .unwrap();
        app.updates(&worker);
        let current = plain(&backend(&mut app, 80, 24));
        assert!(current.contains("The answer stays visible."));
        assert!(app.dialogue.iter().any(|s| s.contains("Source [1]")));
        assert!(app.dialogue.iter().any(|s| s.contains("Diagnostics:")));
        app.focus = Focus::Dialogue;
        app.scroll = u16::MAX;
        assert!(plain(&backend(&mut app, 80, 24)).contains("Diagnostics:"));
    }
    #[test]
    fn shared_geometry_prioritizes_shell_and_normalizes_hidden_controls() {
        use ratatui::layout::Rect;
        let mut app = app();
        for (w, h) in [
            (120, 40),
            (100, 30),
            (80, 24),
            (45, 16),
            (30, 10),
            (10, 5),
            (1, 1),
            (0, 0),
        ] {
            let area = Rect::new(0, 0, w, h);
            let parts = render::areas(area);
            assert!(parts.shell.height >= parts.dialogue.height);
            for rect in [
                parts.shell,
                parts.shell_inner,
                parts.dialogue,
                parts.input,
                parts.footer,
            ] {
                assert!(rect.right() <= w && rect.bottom() <= h);
            }
            backend(&mut app, w, h);
            assert_eq!(
                app.shell.screen().size(),
                (
                    parts.shell_inner.height.max(1),
                    parts.shell_inner.width.max(1)
                )
            );
            app.focus = Focus::Music;
            app.normalize_focus(w, h);
            assert_eq!(app.focus == Focus::Music, render::wide(w, h));
            app.focus = Focus::Dialogue;
            app.normalize_focus(w, h);
            assert_eq!(app.focus == Focus::Dialogue, parts.dialogue.height > 0);
        }
        let tiny = render::areas(Rect::new(0, 0, 1, 1));
        assert_eq!(tiny.shell_inner, Rect::new(0, 0, 1, 1));
    }
    #[test]
    fn shell_first_rectangles_are_contained_disjoint_and_full_height() {
        use ratatui::layout::Rect;
        // Include offset frames and every transition, rather than only origin layouts.
        for w in [0, 1, 30, 79, 80, 99, 100, 120] {
            for h in 0..=42 {
                let frame = Rect::new(7, 5, w, h);
                let a = render::areas(frame);
                let panes = [
                    a.header, a.shell, a.system, a.music, a.dialogue, a.input, a.footer,
                ];
                for (i, pane) in panes.iter().enumerate().filter(|(_, p)| !p.is_empty()) {
                    assert_eq!(pane.intersection(frame), *pane, "{w}x{h}: {pane:?}");
                    for other in &panes[i + 1..] {
                        assert!(
                            pane.intersection(*other).is_empty(),
                            "{w}x{h}: {pane:?} overlaps {other:?}"
                        );
                    }
                }
                assert_eq!(a.shell.intersection(a.shell_inner), a.shell_inner);
                if w >= 100 && h >= 16 {
                    assert_eq!(a.shell.y, a.header.bottom());
                    assert_eq!(a.shell.bottom(), a.footer.y);
                    assert_eq!(a.shell.width, w - 35);
                    assert_eq!(a.dialogue.width, 34);
                    assert_eq!(a.dialogue.bottom(), a.input.y);
                    assert_eq!(a.input.bottom(), a.footer.y);
                    assert!(a.dialogue.x > a.shell.right());
                    if h >= 30 {
                        assert_eq!(a.system.y, a.header.bottom());
                        assert_eq!(a.system.height, 5);
                        assert_eq!(a.system.bottom(), a.music.y);
                        assert_eq!(a.music.height, 5);
                        assert_eq!(a.music.bottom(), a.dialogue.y);
                    } else {
                        assert!(a.system.is_empty() && a.music.is_empty());
                    }
                }
            }
        }
    }
    #[test]
    fn compact_sidebar_preserves_reply_portrait_input_and_visible_focus() {
        let worker = Worker::spawn(|_, _| {}).unwrap();
        let mut app = app();
        app.companion = "Pastel · Uptime is 42 minutes.".into();
        for (w, h) in [(100, 30), (120, 40), (100, 24), (80, 24)] {
            let contents = plain(&backend(&mut app, w, h));
            assert!(contents.contains("Uptime is 42 minutes."), "{w}x{h}");
            assert!(contents.contains("HEART INPUT"), "{w}x{h}");
            assert!(app.portrait_rect.is_some(), "{w}x{h}");
        }
        for (w, h) in [(100, 16), (120, 29), (80, 24), (30, 10)] {
            app.focus = Focus::Music;
            backend(&mut app, w, h);
            assert_eq!(app.focus, Focus::Input);
            // Tab never selects a hidden utility, including in short wide layouts.
            for _ in 0..8 {
                app.key(key(KeyCode::Tab), &worker, w, h);
                assert_ne!(app.focus, Focus::Music);
            }
            app.focus = Focus::Shell;
            app.key(key(KeyCode::F(12)), &worker, w, h);
            assert_eq!(app.focus, Focus::Input);
            assert!(
                render::areas(ratatui::layout::Rect::new(0, 0, w, h))
                    .input
                    .height
                    > 0
            );
        }
    }
    #[test]
    fn shell_cursor_uses_original_vt_coordinates_through_focus_changes() {
        use ratatui::backend::Backend;
        for (w, h) in [(100, 30), (120, 40), (80, 24)] {
            let mut app = app();
            app.preview_terminal(w, h, b"prompt> ");
            app.focus = Focus::Shell;
            let inner = render::areas(ratatui::layout::Rect::new(0, 0, w, h)).shell_inner;
            let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
            for expected in [(0, 8), (3, 6)] {
                if expected.0 == 3 {
                    app.preview_terminal(w, h, b"\x1b[4;7H");
                }
                terminal.draw(|f| draw(f, &mut app)).unwrap();
                let cursor = terminal.backend_mut().get_cursor_position().unwrap();
                assert_eq!(
                    (cursor.x, cursor.y),
                    (inner.x + expected.1, inner.y + expected.0)
                );
                app.focus = Focus::Input;
                terminal.draw(|f| draw(f, &mut app)).unwrap();
                app.focus = Focus::Shell;
                terminal.draw(|f| draw(f, &mut app)).unwrap();
                assert_eq!(
                    terminal.backend_mut().get_cursor_position().unwrap(),
                    cursor
                );
            }
        }
    }
    #[test]
    fn themed_layouts_keep_character_dialogue_and_safe_small_sizes() {
        for id in ["cat", "butler"] {
            for (w, h) in [
                (120, 40),
                (100, 30),
                (100, 31),
                (120, 30),
                (80, 24),
                (45, 16),
                (30, 10),
                (10, 5),
                (1, 1),
                (0, 0),
            ] {
                let mut app = app();
                app.session.selected = id.into();
                let b = backend(&mut app, w, h);
                let contents = plain(&b);
                if w >= 80 {
                    assert!(contents.contains("HEART"));
                    assert!(
                        contents.contains("READY"),
                        "status/header missing at {w}x{h}"
                    );
                    assert!(contents.contains("DIALOGUE"));
                    assert!(contents.contains("Portrait") && contents.contains("unavailable"));
                    assert!(!contents.contains("GOOD IDEAS"));
                    assert!(contents.contains(if id == "cat" { "Pastel" } else { "Butler" }));
                }
                if w >= 100 && h >= 30 {
                    assert!(contents.contains("STATIC DEMO"));
                    assert!(contents.contains("LOCAL MOCK"));
                    for reading in ["CPU   32%", "Memory   9.3 / 16 GB", "Disk   322 / 476 GB"] {
                        assert!(
                            contents.contains(reading),
                            "missing {reading} for {id} at {w}x{h}"
                        );
                    }
                } else {
                    assert!(!contents.contains("STATIC DEMO"));
                }
            }
        }
        let mut app = app();
        let cat = backend(&mut app, 120, 40);
        app.session.selected = "butler".into();
        let butler = backend(&mut app, 120, 40);
        assert_ne!(cat.buffer(), butler.buffer());
        app.session.profile();
    }
    #[test]
    fn unicode_editing_history_and_focus_follow_visible_panels() {
        let worker = Worker::spawn(|_, _| {}).unwrap();
        let mut app = app();
        for c in "q café猫🙂".chars() {
            assert!(!app.key(key(KeyCode::Char(c)), &worker, 120, 40));
        }
        assert_eq!(app.input, "q café猫🙂");
        app.key(key(KeyCode::Left), &worker, 120, 40);
        app.key(key(KeyCode::Backspace), &worker, 120, 40);
        app.key(key(KeyCode::Delete), &worker, 120, 40);
        assert_eq!(app.input, "q café");
        assert!(app.input.is_char_boundary(app.cursor));
        app.key(key(KeyCode::Home), &worker, 120, 40);
        app.key(key(KeyCode::Delete), &worker, 120, 40);
        assert_eq!(app.input, " café");
        app.key(key(KeyCode::Tab), &worker, 120, 40);
        assert_eq!(app.focus, Focus::Dialogue);
        app.key(key(KeyCode::Tab), &worker, 120, 40);
        assert_eq!(app.focus, Focus::Music);
        app.key(key(KeyCode::Char(' ')), &worker, 120, 40);
        assert!(app.playing);
        app.key(key(KeyCode::Right), &worker, 120, 40);
        assert_eq!(app.track, 1);
        app.normalize_focus(80, 24);
        assert_eq!(app.focus, Focus::Input);
        app.input = "/help".into();
        app.cursor = 5;
        app.submit(&worker);
        assert!(app.help);
        app.key(key(KeyCode::Esc), &worker, 80, 24);
        app.key(key(KeyCode::Up), &worker, 80, 24);
        assert_eq!(app.input, "/help");
        app.key(key(KeyCode::Down), &worker, 80, 24);
        assert!(app.input.is_empty());
        app.key(key(KeyCode::BackTab), &worker, 80, 24);
        assert_eq!(app.focus, Focus::Shell);
    }
    #[test]
    fn long_content_scrolls_help_and_input_remain_bounded() {
        let worker = Worker::spawn(|_, _| {}).unwrap();
        let mut app = app();
        app.dialogue = vec!["猫 café source https://example.com/long ".repeat(400)];
        app.input = "é猫🙂".repeat(600);
        app.cursor = app.input.len();
        app.focus = Focus::Dialogue;
        let b = backend(&mut app, 80, 24);
        assert!(plain(&b).contains("HEART INPUT"));
        assert!(app.scroll > 0);
        app.focus = Focus::Dialogue;
        app.key(key(KeyCode::Home), &worker, 80, 24);
        backend(&mut app, 80, 24);
        assert_eq!(app.scroll, 0);
        app.help = true;
        backend(&mut app, 30, 10);
        app.key(key(KeyCode::End), &worker, 30, 10);
        backend(&mut app, 30, 10);
        assert!(app.help_scroll > 0);
        for (w, h) in [(120, 40), (80, 24), (30, 10), (1, 1), (0, 0)] {
            backend(&mut app, w, h);
        }
    }
    #[test]
    fn kitty_placements_follow_help_profile_and_resize_without_idle_transmission() {
        let mut app = app();
        app.portrait = portrait::Portrait::fixture();
        let home = backend(&mut app, 120, 40);
        let rect = app.portrait_rect.unwrap();
        let symbols = plain(&home);
        assert!(symbols.contains('\u{10eeee}'));
        assert!(symbols.contains("\x1b_G"));
        assert!(!plain(&backend(&mut app, 120, 40)).contains("\x1b_G"));
        app.help = true;
        let help = backend(&mut app, 120, 40);
        assert!(!plain(&help).contains('\u{10eeee}'));
        assert!(app.portrait_rect.is_none());
        app.help = false;
        assert!(plain(&backend(&mut app, 120, 40)).contains('\u{10eeee}'));
        app.session.selected = "butler".into();
        let butler = plain(&backend(&mut app, 120, 40));
        assert!(!butler.contains('\u{10eeee}'));
        assert!(butler.contains("Text mode"));
        app.session.selected = "cat".into();
        let compact = plain(&backend(&mut app, 45, 16));
        assert!(!compact.contains('\u{10eeee}'));
        assert!(app.portrait_rect.is_none());
        let resized = plain(&backend(&mut app, 80, 24));
        assert!(resized.contains('\u{10eeee}'));
        assert!(resized.contains("\x1b_G"));
        assert_ne!(app.portrait_rect.unwrap(), rect);
        assert!(!plain(&backend(&mut app, 80, 24)).contains("\x1b_G"));
    }
    #[test]
    fn appearance_names_label_snapshots_and_old_custom_profiles_still_render() {
        let mut app = app();
        assert!(app.dialogue[0].starts_with("Pastel · "));
        let captured = speaker(app.session.profile()).to_owned();
        app.switch_profile();
        assert_eq!(captured, "Pastel");
        assert!(app.dialogue.last().unwrap().starts_with("Butler · "));
        let (requests, _) = mpsc::sync_channel(1);
        let (send, updates) = mpsc::channel();
        let worker = Worker { requests, updates };
        send.send(Update::Progress(captured.clone(), "working".into()))
            .unwrap();
        send.send(Update::Done(captured, vec!["done".into()]))
            .unwrap();
        app.updates(&worker);
        assert!(app.dialogue.iter().any(|s| s == "Pastel · working"));
        assert!(app.dialogue.last().unwrap() == "Pastel · done");
        let mut custom = Personality::cat();
        custom.id = "custom".into();
        custom.appearance = None;
        assert_eq!(speaker(&custom), "custom");
        app.session.profiles.insert("custom".into(), custom);
        app.session.selected = "custom".into();
        let contents = plain(&backend(&mut app, 80, 24));
        assert!(contents.contains("custom"));
        assert!(contents.contains("Portrait") && contents.contains("unavailable"));
    }
    #[test]
    fn long_input_reserves_a_visible_caret_after_ascii_and_wide_unicode() {
        use ratatui::backend::Backend;
        for value in ["a".repeat(76), "é猫🙂".repeat(30)] {
            let mut app = app();
            app.input = value;
            app.cursor = app.input.len();
            let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
            terminal.draw(|f| draw(f, &mut app)).unwrap();
            let caret = terminal.backend_mut().get_cursor_position().unwrap();
            assert!(caret.x < 79);
            assert_eq!(
                terminal.backend().buffer()[(caret.x, caret.y)].symbol(),
                " "
            );
            if app.input.starts_with('a') {
                assert_eq!(caret.x, 78);
                assert_eq!(
                    terminal.backend().buffer()[(caret.x - 1, caret.y)].symbol(),
                    "a"
                );
            }
        }
    }
    #[test]
    fn slow_worker_does_not_block_edits_switch_help_or_quit_and_keeps_request_profile() {
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let worker = Worker::spawn(move |work, updates| {
            started_tx.send(work.profile.id.clone()).unwrap();
            release_rx.recv().unwrap();
            let _ = updates.send(Update::Done(
                work.profile.id,
                vec!["finished original request".into()],
            ));
        })
        .unwrap();
        let mut app = app();
        app.input = "/disk /".into();
        app.cursor = app.input.len();
        app.submit(&worker);
        assert_eq!(
            started_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            "cat"
        );
        assert!(app.busy);
        app.key(key(KeyCode::F(2)), &worker, 120, 40);
        assert_eq!(app.session.selected, "butler");
        for c in "/uptime".chars() {
            app.key(key(KeyCode::Char(c)), &worker, 120, 40);
        }
        app.submit(&worker);
        assert_eq!(app.input, "/uptime"); // second request is not queued
        app.key(key(KeyCode::F(1)), &worker, 120, 40);
        assert!(app.help);
        backend(&mut app, 80, 24);
        assert!(app.key(
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL),
            &worker,
            120,
            40
        ));
        release_tx.send(()).unwrap();
        let update = worker.updates.recv_timeout(Duration::from_secs(1)).unwrap();
        match update {
            Update::Done(name, lines) => {
                assert_eq!(name, "cat");
                assert_eq!(lines, ["finished original request"]);
            }
            _ => panic!("expected reply"),
        }
        // Dropping while a task is pending never joins it.
        let (release_tx, release_rx) = mpsc::channel();
        let worker = Worker::spawn(move |_, _| {
            let _ = release_rx.recv();
        })
        .unwrap();
        worker
            .requests
            .send(Work {
                request: "/disk".into(),
                profile: Personality::cat(),
            })
            .unwrap();
        drop(worker);
        release_tx.send(()).unwrap();
    }
}
