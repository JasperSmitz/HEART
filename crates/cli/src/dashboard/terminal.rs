//! Manual terminal only. No engine, model, module, or request worker has an input handle.
//! portable-pty owns PTY creation/setsid/TIOCSCTTY; vt100 owns escape parsing.
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::{
    io::{self, Read, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use tui_term::vt100::{self, Callbacks};

const SCROLLBACK: usize = 2000;
const CHUNK: usize = 4096;
const FRAME_CHUNKS: usize = 16;
const INPUT_LIMIT: usize = 64 * 1024;
const RETRY: Duration = Duration::from_millis(4);

/// Unsupported OSC/DCS/APC/PM strings have no screen-cell meaning here. Drop
/// them incrementally before vte: its std OSC buffer otherwise grows until ST.
/// This is a bounded string guard, not a second terminal parser.
#[derive(Default)]
enum StringGuard {
    #[default]
    Ground,
    Escape,
    String {
        osc: bool,
        escape: bool,
    },
}
impl StringGuard {
    fn filter(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut output = Vec::with_capacity(bytes.len() + 1);
        for &byte in bytes {
            match self {
                Self::Ground if byte == 0x1b => *self = Self::Escape,
                Self::Ground => output.push(byte),
                Self::Escape => match byte {
                    b']' | b'P' | b'_' | b'^' => {
                        *self = Self::String {
                            osc: byte == b']',
                            escape: false,
                        }
                    }
                    0x1b => {}
                    _ => {
                        output.extend_from_slice(&[0x1b, byte]);
                        *self = Self::Ground;
                    }
                },
                Self::String { osc, escape } => {
                    if (*escape && byte == b'\\')
                        || (*osc && byte == 7)
                        || matches!(byte, 0x18 | 0x1a)
                    {
                        *self = Self::Ground;
                    } else {
                        *escape = byte == 0x1b;
                    }
                }
            }
        }
        output
    }
}

#[derive(Default)]
struct Reports {
    replies: Vec<u8>,
    hvp: Option<(u16, u16)>,
}
impl Callbacks for Reports {
    fn unhandled_csi(
        &mut self,
        screen: &mut vt100::Screen,
        i1: Option<u8>,
        i2: Option<u8>,
        params: &[&[u16]],
        c: char,
    ) {
        if i2.is_some() {
            return;
        }
        // vt100 0.16 supports CUP but omits the standard equivalent HVP.
        // The existing parser identifies this CSI; re-enter it with canonical CUP
        // before processing subsequent text. No raw-sequence recognition here.
        if i1.is_none() && c == 'f' {
            let value = |n: usize| {
                params
                    .get(n)
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1)
                    .max(1)
            };
            self.hvp = Some((value(0), value(1)));
            return;
        }
        let p = params.first().and_then(|p| p.first()).copied().unwrap_or(0);
        let reply = match (i1, c, p) {
            // VT100 with ANSI color, no graphics/clipboard/outer-terminal extensions.
            (None, 'c', 0) => "\x1b[?1;2c".into(),
            (None, 'n', 5) => "\x1b[0n".into(),
            (None | Some(b'?'), 'n', 6) => {
                let (row, col) = screen.cursor_position();
                format!(
                    "\x1b[{}{};{}R",
                    if i1.is_some() { "?" } else { "" },
                    row + 1,
                    col + 1
                )
            }
            (None, 't', 18) => {
                let (r, c) = screen.size();
                format!("\x1b[8;{r};{c}t")
            }
            _ => return,
        };
        // A hostile query flood cannot grow a frame's reply buffer without bound.
        if self.replies.len() + reply.len() <= CHUNK {
            self.replies.extend_from_slice(reply.as_bytes());
        }
    }
}

enum Output {
    Bytes(Vec<u8>),
    Eof,
    Error(String),
}
struct Running {
    master: Box<dyn MasterPty + Send>,
    child: Option<Box<dyn Child + Send + Sync>>,
    pid: Option<u32>,
    input: Option<SyncSender<Vec<u8>>>,
    output: Receiver<Output>,
    stop: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

/// Screen state and bounded I/O owned exclusively by the dashboard.
pub(super) struct TerminalPane {
    parser: vt100::Parser<Reports>,
    strings: StringGuard,
    running: Option<Running>,
    status: String,
    pub notice: Option<String>,
    ended: bool,
}

pub(super) fn shell_command(shell: impl AsRef<std::ffi::OsStr>) -> io::Result<CommandBuilder> {
    let mut cmd = CommandBuilder::new(shell);
    cmd.arg("-i");
    cmd.cwd(std::env::current_dir()?);
    cmd.env("TERM", "xterm-256color");
    // Keep PATH, ZDOTDIR, Starship and all normal rc/environment configuration.
    for key in [
        "TERM_PROGRAM",
        "TERM_PROGRAM_VERSION",
        "TERMINFO",
        "TERMINFO_DIRS",
        "GHOSTTY_RESOURCES_DIR",
        "GHOSTTY_BIN_DIR",
        "GHOSTTY_SHELL_INTEGRATION",
        "KITTY_WINDOW_ID",
        "KITTY_PID",
        "WINDOWID",
        "WEZTERM_PANE",
        "WT_SESSION",
        "TMUX",
        "TMUX_PANE",
        "STY",
        "LINES",
        "COLUMNS",
    ] {
        cmd.env_remove(key);
    }
    Ok(cmd)
}

impl TerminalPane {
    /// Preview construction never opens a PTY or spawns a process.
    pub fn preview() -> Self {
        Self {
            parser: vt100::Parser::new_with_callbacks(1, 1, SCROLLBACK, Reports::default()),
            strings: StringGuard::default(),
            running: None,
            status: "PREVIEW · no shell".into(),
            notice: None,
            ended: false,
        }
    }
    pub fn spawn(rows: u16, cols: u16) -> Self {
        let mut pane = Self::preview();
        pane.resize(rows, cols);
        let result = std::env::var_os("SHELL")
            .filter(|s| !s.is_empty())
            .ok_or_else(|| io::Error::other("SHELL is not configured"))
            .and_then(shell_command)
            .and_then(|cmd| Running::spawn(cmd, rows, cols));
        match result {
            Ok(running) => {
                pane.running = Some(running);
                pane.status = "LIVE".into();
            }
            Err(error) => {
                pane.status = format!("START FAILED · {error}");
                pane.ended = true;
            }
        }
        pane
    }
    pub fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }
    pub fn ended(&self) -> bool {
        self.ended
    }
    pub fn label(&self) -> String {
        let offset = self.screen().scrollback();
        if offset == 0 {
            self.status.clone()
        } else {
            format!("SCROLLBACK {offset} · Shift-PgDn to live")
        }
    }
    pub fn preview_output(&mut self, bytes: &[u8]) {
        self.process_output(bytes);
    }
    fn process_output(&mut self, bytes: &[u8]) {
        let filtered = self.strings.filter(bytes);
        // Each HVP ends in 'f'. Splitting here lets the parser apply its alias
        // at the exact sequence boundary, including CSI fragmented across reads.
        // Literal 'f' just passes through; strings are suppressed by StringGuard.
        for part in filtered.split_inclusive(|b| *b == b'f') {
            self.parser.process(part);
            if let Some((row, col)) = self.parser.callbacks_mut().hvp.take() {
                self.parser.process(format!("\x1b[{row};{col}H").as_bytes());
            }
        }
    }
    pub fn resize(&mut self, rows: u16, cols: u16) {
        let size = (rows.max(1), cols.max(1));
        if self.screen().size() == size {
            return;
        }
        self.parser.screen_mut().set_size(size.0, size.1);
        if let Some(running) = &self.running
            && let Err(error) = running.master.resize(pty_size(size.0, size.1))
        {
            self.notice = Some(format!("Resize failed: {error}"));
        }
    }
    pub fn drain(&mut self) {
        for _ in 0..FRAME_CHUNKS {
            let Some(running) = &self.running else {
                break;
            };
            match running.output.try_recv() {
                Ok(Output::Bytes(bytes)) => self.process_output(&bytes),
                Ok(Output::Eof) => {
                    self.ended = true;
                    self.status = "OUTPUT CLOSED · F12 for HEART".into();
                    break;
                }
                Ok(Output::Error(error)) => {
                    self.ended = true;
                    self.status = format!("I/O ERROR · {error}");
                    break;
                }
                Err(_) => break,
            }
        }
        let reports = std::mem::take(&mut self.parser.callbacks_mut().replies);
        if !reports.is_empty() {
            self.send(reports);
        }
        if let Some(running) = &mut self.running
            && let Some(child) = &mut running.child
        {
            match child.try_wait() {
                Ok(Some(status)) => {
                    running.child = None;
                    self.ended = true;
                    self.status = format!("EXITED ({status}) · F12 for HEART");
                }
                Err(error) => self.notice = Some(format!("Shell wait failed: {error}")),
                _ => {}
            }
        }
    }
    fn send(&mut self, bytes: Vec<u8>) {
        if bytes.is_empty() {
            return;
        }
        let Some(input) = self.running.as_ref().and_then(|r| r.input.as_ref()) else {
            return;
        };
        if self.ended {
            return;
        }
        self.notice = match input.try_send(bytes) {
            Ok(()) => None,
            Err(TrySendError::Full(_)) => {
                Some("Input busy: event rejected; retry after child resumes reading.".into())
            }
            Err(TrySendError::Disconnected(_)) => Some("Shell input closed · F12 for HEART".into()),
        };
    }
    pub fn key(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        if key.modifiers == KeyModifiers::SHIFT
            && matches!(key.code, KeyCode::PageUp | KeyCode::PageDown)
        {
            let offset = self.screen().scrollback();
            let page = usize::from(self.screen().size().0.saturating_sub(1).max(1));
            self.parser
                .screen_mut()
                .set_scrollback(if key.code == KeyCode::PageUp {
                    offset.saturating_add(page)
                } else {
                    offset.saturating_sub(page)
                });
            return;
        }
        let bytes = encode_key(key, self.screen().application_cursor());
        if !bytes.is_empty() {
            self.parser.screen_mut().set_scrollback(0);
            self.send(bytes);
        }
    }
    pub fn paste(&mut self, text: &str) {
        // Reject oversized events atomically; never truncate a bracketed-paste terminator.
        if text.len() > INPUT_LIMIT {
            self.notice = Some("Paste rejected: limit is 64 KiB.".into());
            return;
        }
        let text = text.replace("\r\n", "\n").replace('\r', "\n");
        // Embedded ESC cannot escape bracketed paste and execute a pasted newline.
        let text = text.replace('\x1b', "");
        let bytes = if self.screen().bracketed_paste() {
            format!("\x1b[200~{text}\x1b[201~").into_bytes()
        } else {
            text.replace('\n', "\r").into_bytes()
        };
        self.parser.screen_mut().set_scrollback(0);
        self.send(bytes);
    }
}

fn pty_size(rows: u16, cols: u16) -> PtySize {
    PtySize {
        rows: rows.max(1),
        cols: cols.max(1),
        pixel_width: 0,
        pixel_height: 0,
    }
}
fn retry_send<T>(tx: &SyncSender<T>, mut value: T, stop: &AtomicBool) -> bool {
    while !stop.load(Ordering::Relaxed) {
        match tx.try_send(value) {
            Ok(()) => return true,
            Err(TrySendError::Full(v)) => {
                value = v;
                thread::sleep(RETRY);
            }
            Err(TrySendError::Disconnected(_)) => return false,
        }
    }
    false
}
impl Running {
    fn spawn(cmd: CommandBuilder, rows: u16, cols: u16) -> io::Result<Self> {
        let pair = native_pty_system()
            .openpty(pty_size(rows, cols))
            .map_err(io::Error::other)?;
        // Nonblocking applies to the duplicated handles as well. All reads/writes retry
        // on background threads, so cancellation can close handles even if a child stalls.
        #[cfg(unix)]
        {
            use nix::fcntl::{FcntlArg, OFlag, fcntl};
            let fd = pair
                .master
                .as_raw_fd()
                .ok_or_else(|| io::Error::other("PTY has no file descriptor"))?;
            let flags = fcntl(fd, FcntlArg::F_GETFL).map_err(io::Error::other)?;
            fcntl(
                fd,
                FcntlArg::F_SETFL(OFlag::from_bits_truncate(flags) | OFlag::O_NONBLOCK),
            )
            .map_err(io::Error::other)?;
        }
        let mut reader = pair.master.try_clone_reader().map_err(io::Error::other)?;
        // portable-pty's Unix writer destructor sends a newline plus VEOF. That
        // can execute an unsubmitted command during shutdown. A duplicated File
        // retains nonblocking I/O but its destructor ONLY closes the descriptor.
        #[cfg(unix)]
        let mut writer: Box<dyn Write + Send> = {
            use std::os::fd::BorrowedFd;
            let fd = pair
                .master
                .as_raw_fd()
                .ok_or_else(|| io::Error::other("PTY has no file descriptor"))?;
            // SAFETY: pair.master owns this live fd and remains alive throughout
            // the borrow and duplication. The resulting OwnedFd is independent;
            // no borrowed fd escapes this expression or outlives the master.
            let owned = unsafe { BorrowedFd::borrow_raw(fd) }.try_clone_to_owned()?;
            Box::new(std::fs::File::from(owned))
        };
        #[cfg(not(unix))]
        let mut writer = pair.master.take_writer().map_err(io::Error::other)?;
        let child = pair.slave.spawn_command(cmd).map_err(io::Error::other)?;
        drop(pair.slave);
        let (input, rx) = mpsc::sync_channel::<Vec<u8>>(16);
        let (tx, output) = mpsc::sync_channel(32);
        let stop = Arc::new(AtomicBool::new(false));
        let mut running = Self {
            pid: child.process_id(),
            master: pair.master,
            child: Some(child),
            input: Some(input),
            output,
            stop,
            threads: vec![],
        };
        let stopped = running.stop.clone();
        let errors = tx.clone();
        running.threads.push(
            thread::Builder::new()
                .name("heart-pty-reader".into())
                .spawn(move || {
                    let mut buf = [0; CHUNK];
                    while !stopped.load(Ordering::Relaxed) {
                        let output = match reader.read(&mut buf) {
                            Ok(0) => Output::Eof,
                            Ok(n) => Output::Bytes(buf[..n].to_vec()),
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                thread::sleep(RETRY);
                                continue;
                            }
                            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                            // Linux PTYs return EIO once all slave descriptors close.
                            Err(e) if e.raw_os_error() == Some(5) => Output::Eof,
                            Err(e) => Output::Error(e.to_string()),
                        };
                        let done = !matches!(output, Output::Bytes(_));
                        if !retry_send(&tx, output, &stopped) || done {
                            break;
                        }
                    }
                })?,
        );
        let stopped = running.stop.clone();
        running.threads.push(
            thread::Builder::new()
                .name("heart-pty-writer".into())
                .spawn(move || {
                    while !stopped.load(Ordering::Relaxed) {
                        let bytes = match rx.recv_timeout(RETRY) {
                            Ok(bytes) => bytes,
                            Err(mpsc::RecvTimeoutError::Timeout) => continue,
                            Err(_) => break,
                        };
                        let mut rest = bytes.as_slice();
                        while !rest.is_empty() && !stopped.load(Ordering::Relaxed) {
                            match writer.write(rest) {
                                Ok(0) => {
                                    let _ = retry_send(
                                        &errors,
                                        Output::Error("PTY write returned zero".into()),
                                        &stopped,
                                    );
                                    return;
                                }
                                Ok(n) => rest = &rest[n..],
                                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                    thread::sleep(RETRY)
                                }
                                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                                Err(e) => {
                                    let _ =
                                        retry_send(&errors, Output::Error(e.to_string()), &stopped);
                                    return;
                                }
                            }
                        }
                    }
                })?,
        );
        Ok(running)
    }
}

#[cfg(unix)]
fn owned_group(group: i32, session: i32) -> bool {
    use nix::unistd::{Pid, getpgid, getpgrp, getsid};
    group > 1
        && group != getpgrp().as_raw()
        && getsid(Some(Pid::from_raw(group))).is_ok_and(|s| s.as_raw() == session)
        && getpgid(Some(Pid::from_raw(group))).is_ok_and(|g| g.as_raw() == group)
}
#[cfg(unix)]
fn attached_foreground(master: &dyn MasterPty, group: i32) -> bool {
    // tcsetpgrp only permits a process group in the controlling terminal's
    // session. Query OUR owned PTY each time: the group leader need not exist.
    // Never retain an unchecked PGID after the terminal loses its attachment.
    group > 1
        && group != nix::unistd::getpgrp().as_raw()
        && master.process_group_leader() == Some(group)
}
impl Drop for Running {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.input.take();
        #[cfg(unix)]
        if let Some(pid) = self.pid.and_then(|p| i32::try_from(p).ok()) {
            use nix::{
                sys::signal::{Signal, killpg},
                unistd::Pid,
            };
            // setsid in portable-pty makes the spawned child the session and shell
            // group leader. tcgetpgrp is ONLY the attached foreground job group.
            // Finish the foreground job BEFORE terminating its session leader.
            // Otherwise shell exit can detach the controlling terminal before
            // escalation, removing the kernel proof that this group is ours.
            if let Some(foreground) = self.master.process_group_leader().filter(|g| *g != pid) {
                for signal in [Signal::SIGHUP, Signal::SIGTERM, Signal::SIGKILL] {
                    if !attached_foreground(self.master.as_ref(), foreground) {
                        break;
                    }
                    let _ = killpg(Pid::from_raw(foreground), signal);
                    let deadline = Instant::now() + Duration::from_millis(100);
                    while Instant::now() < deadline
                        && attached_foreground(self.master.as_ref(), foreground)
                    {
                        thread::sleep(RETRY);
                    }
                }
            }
            for signal in [Signal::SIGHUP, Signal::SIGTERM, Signal::SIGKILL] {
                if owned_group(pid, pid) {
                    let _ = killpg(Pid::from_raw(pid), signal);
                }
                let deadline = Instant::now() + Duration::from_millis(100);
                while Instant::now() < deadline {
                    if let Some(child) = &mut self.child
                        && matches!(child.try_wait(), Ok(Some(_)))
                    {
                        self.child = None;
                    }
                    if self.child.is_none() {
                        break;
                    }
                    thread::sleep(RETRY);
                }
            }
        }
        // Never wait indefinitely on an uninterruptible child. A detached reaper
        // retains the owned handle if SIGKILL has not yet become waitable.
        if let Some(mut child) = self.child.take() {
            let _ = thread::Builder::new()
                .name("heart-pty-reap".into())
                .spawn(move || {
                    let _ = child.wait();
                });
        }
        for handle in self.threads.drain(..) {
            if handle.is_finished() {
                let _ = handle.join();
            }
        }
    }
}

fn encode_key(key: KeyEvent, application: bool) -> Vec<u8> {
    use KeyCode::*;
    if key.kind == KeyEventKind::Release {
        return vec![];
    }
    let mods = key.modifiers;
    let modifier = 1
        + u8::from(mods.contains(KeyModifiers::SHIFT))
        + 2 * u8::from(mods.contains(KeyModifiers::ALT))
        + 4 * u8::from(mods.contains(KeyModifiers::CONTROL));
    let special = |suffix: char| {
        if modifier > 1 {
            format!("\x1b[1;{modifier}{suffix}")
        } else {
            format!("\x1b{}{suffix}", if application { 'O' } else { '[' })
        }
    };
    let tilde = |n| {
        if modifier > 1 {
            format!("\x1b[{n};{modifier}~")
        } else {
            format!("\x1b[{n}~")
        }
    };
    let text = match key.code {
        Up => special('A'),
        Down => special('B'),
        Right => special('C'),
        Left => special('D'),
        Home => special('H'),
        End => special('F'),
        Insert => tilde(2),
        Delete => tilde(3),
        PageUp => tilde(5),
        PageDown => tilde(6),
        F(n @ 1..=4) => {
            let c = char::from(b'P' + n - 1);
            if modifier > 1 {
                format!("\x1b[1;{modifier}{c}")
            } else {
                format!("\x1bO{c}")
            }
        }
        F(n @ 5..=12) => tilde([15, 17, 18, 19, 20, 21, 23, 24][usize::from(n - 5)]),
        BackTab => "\x1b[Z".into(),
        _ => {
            let mut s = match key.code {
                Enter => "\r".into(),
                Tab => "\t".into(),
                Backspace => "\x7f".into(),
                Esc => "\x1b".into(),
                Null => "\0".into(),
                Char(c) if mods.contains(KeyModifiers::CONTROL) => match c.to_ascii_lowercase() {
                    'a'..='z' => ((c.to_ascii_lowercase() as u8 - b'a' + 1) as char).to_string(),
                    ' ' | '@' | '2' => "\0".into(),
                    '[' | '3' => "\x1b".into(),
                    '\\' | '4' => "\x1c".into(),
                    ']' | '5' => "\x1d".into(),
                    '^' | '6' => "\x1e".into(),
                    '_' | '7' => "\x1f".into(),
                    '?' | '8' => "\x7f".into(),
                    _ => return vec![],
                },
                Char(c) => c.to_string(),
                _ => return vec![],
            };
            if mods.contains(KeyModifiers::ALT) {
                s.insert(0, '\x1b');
            }
            s
        }
    };
    text.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

    #[test]
    fn keys_preserve_shell_chords_application_cursor_and_modifiers() {
        for (code, mods, expected) in [
            (KeyCode::Char('c'), KeyModifiers::CONTROL, "\x03"),
            (KeyCode::Char('q'), KeyModifiers::CONTROL, "\x11"),
            (KeyCode::Char('z'), KeyModifiers::CONTROL, "\x1a"),
            (KeyCode::Tab, KeyModifiers::NONE, "\t"),
            (KeyCode::F(1), KeyModifiers::NONE, "\x1bOP"),
            (KeyCode::F(2), KeyModifiers::NONE, "\x1bOQ"),
            (KeyCode::Char('猫'), KeyModifiers::ALT, "\x1b猫"),
            (
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                "\x1b[1;6D",
            ),
            (KeyCode::Delete, KeyModifiers::ALT, "\x1b[3;3~"),
            (KeyCode::BackTab, KeyModifiers::SHIFT, "\x1b[Z"),
        ] {
            assert_eq!(
                encode_key(KeyEvent::new(code, mods), true),
                expected.as_bytes()
            );
        }
        assert_eq!(
            encode_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), true),
            b"\x1bOA"
        );
        assert_eq!(
            encode_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), false),
            b"\x1b[A"
        );
        let mut release = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        assert!(encode_key(release, false).is_empty());
    }

    #[test]
    fn parser_handles_styles_alternate_screen_reports_and_contains_outer_controls() {
        let mut pane = TerminalPane::preview();
        pane.resize(6, 40);
        pane.preview_output(b"\x1b[1;31mred\x1b[0m\x1b[?1h\x1b[?2004h\x1b[?25l");
        assert!(pane.screen().cell(0, 0).unwrap().bold());
        assert_eq!(
            pane.screen().cell(0, 0).unwrap().fgcolor(),
            vt100::Color::Idx(1)
        );
        assert!(pane.screen().application_cursor());
        assert!(pane.screen().bracketed_paste());
        assert!(pane.screen().hide_cursor());
        pane.preview_output(b"\x1b[?1049h\x1b[Halt");
        assert!(pane.screen().alternate_screen());
        assert!(pane.screen().contents().contains("alt"));
        pane.preview_output(b"\x1b[?1049l\x1b[2;4H\x1b[6n\x1b[5n\x1b[c\x1b[18t");
        assert!(!pane.screen().alternate_screen());
        assert!(pane.screen().contents().contains("red"));
        assert_eq!(
            pane.parser.callbacks().replies,
            b"\x1b[2;4R\x1b[0n\x1b[?1;2c\x1b[8;6;40t"
        );
        for byte in b"\x1b]52;c;c2VjcmV0\x07\x1b]2;host-title\x1b\\\x1b_Ga=T;secret\x1b\\" {
            pane.preview_output(&[*byte]);
        }
        let area = Rect::new(0, 0, 40, 6);
        let mut buffer = Buffer::empty(area);
        tui_term::widget::PseudoTerminal::new(pane.screen()).render(area, &mut buffer);
        let rendered: String = buffer.content().iter().map(|c| c.symbol()).collect();
        assert!(!rendered.contains('\x1b'));
        assert!(!rendered.contains("host-title"));
        assert!(!rendered.contains("secret"));
    }

    #[test]
    fn scrollback_is_bounded_and_regular_input_returns_to_live() {
        let mut pane = TerminalPane::preview();
        pane.resize(4, 30);
        for n in 0..2500 {
            pane.preview_output(format!("line {n}\r\n").as_bytes());
        }
        for _ in 0..1000 {
            pane.key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::SHIFT));
        }
        assert_eq!(pane.screen().scrollback(), SCROLLBACK);
        assert!(!pane.screen().contents().contains("line 0\n"));
        pane.key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        assert_eq!(pane.screen().scrollback(), 0);
        assert!(pane.screen().contents().contains("line 2499"));
    }

    #[test]
    fn hvp_alias_preserves_position_order_and_fragmented_csi_without_rewriting_strings() {
        let stream = b"\x1b[2;3fA\x1b[4;8fB\x1b[0;0fC\x1b[3;1Hfood\x1b]2;\x1b[6;20fhost-title\x07D\x1bP\x1b[6;20ff\x1b\\E";
        // Every possible read boundary, and one-byte reads, must yield the same
        // positions. OSC/DCS payload 'f' never creates an HVP action.
        for split in 0..=stream.len() {
            let mut pane = TerminalPane::preview();
            pane.resize(6, 30);
            pane.preview_output(&stream[..split]);
            pane.preview_output(&stream[split..]);
            for (row, col, text) in [
                (1, 2, "A"),
                (3, 7, "B"),
                (0, 0, "C"),
                (2, 0, "f"),
                (2, 4, "D"),
                (2, 5, "E"),
            ] {
                assert_eq!(
                    pane.screen().cell(row, col).unwrap().contents(),
                    text,
                    "split {split}"
                );
            }
            assert_eq!(pane.screen().cell(5, 19).unwrap().contents(), "");
        }
        let mut pane = TerminalPane::preview();
        pane.resize(6, 30);
        for byte in stream {
            pane.preview_output(&[*byte]);
        }
        assert_eq!(pane.screen().cell(3, 7).unwrap().contents(), "B");
        assert_eq!(pane.screen().cell(2, 5).unwrap().contents(), "E");
    }

    #[test]
    fn unterminated_control_strings_are_discarded_without_buffering_payload() {
        let mut pane = TerminalPane::preview();
        pane.resize(4, 30);
        for introducer in [b']', b'P', b'_', b'^'] {
            pane.preview_output(&[0x1b]);
            pane.preview_output(&[introducer]);
            for _ in 0..1024 {
                let filtered = pane.strings.filter(&[b'x'; CHUNK]);
                assert!(filtered.is_empty());
            }
            pane.preview_output(b"\x1b");
            pane.preview_output(b"\\");
            pane.preview_output("猫é".as_bytes());
        }
        assert_eq!(pane.screen().contents(), "猫é猫é猫é猫é");
        pane.preview_output(b"\x1b[31mR");
        assert_eq!(
            pane.screen().cell(0, 12).unwrap().fgcolor(),
            vt100::Color::Idx(1)
        );
    }

    #[cfg(unix)]
    struct Fixture {
        dir: std::path::PathBuf,
    }
    #[cfg(unix)]
    impl Fixture {
        fn new() -> Self {
            let dir = std::env::temp_dir().join(format!(
                "heart-pty-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&dir).unwrap();
            std::fs::write(dir.join(".zshrc"), "PROMPT='HEART_FIXTURE> '; RPROMPT=''; unsetopt zle_beep; bindkey -e; alias heartalias='printf ALIAS_OK'; compinit() {}; export RC_MARKER=rc_loaded\n").unwrap();
            Self { dir }
        }
        fn pane(&self) -> TerminalPane {
            let mut cmd = shell_command("/bin/zsh").unwrap();
            cmd.env("ZDOTDIR", &self.dir);
            cmd.cwd(&self.dir);
            let mut pane = TerminalPane::preview();
            pane.resize(12, 80);
            pane.running =
                Some(Running::spawn(cmd, 12, 80).expect("real PTY required; do not silently skip"));
            pane.status = "LIVE".into();
            until(&mut pane, |p| {
                p.screen().contents().contains("HEART_FIXTURE>")
            });
            pane
        }
    }
    #[cfg(unix)]
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }
    #[cfg(unix)]
    fn until(pane: &mut TerminalPane, mut predicate: impl FnMut(&TerminalPane) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            pane.drain();
            if predicate(pane) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "PTY condition timed out: {} / {}",
                pane.status,
                pane.screen().contents()
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    #[cfg(unix)]
    #[test]
    fn real_zsh_rc_history_completion_interrupt_resize_and_eof() {
        let fixture = Fixture::new();
        std::fs::write(fixture.dir.join("completion_unique_file"), "").unwrap();
        let mut pane = fixture.pane();
        pane.send(b"heartalias; printf ':%s:%s\\n' $RC_MARKER $TERM\r".to_vec());
        until(&mut pane, |p| {
            p.screen()
                .contents()
                .contains("ALIAS_OK:rc_loaded:xterm-256color")
        });
        pane.send(b"printf HISTORY_%s RAN\r".to_vec());
        until(&mut pane, |p| p.screen().contents().contains("HISTORY_RAN"));
        // Clear the screen so a second actual result (not command echo) proves history.
        pane.preview_output(b"\x1b[2J\x1b[H");
        pane.key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        pane.key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        until(&mut pane, |p| p.screen().contents().contains("HISTORY_RAN"));
        pane.send(b"cat completion_uni".to_vec());
        pane.key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        until(&mut pane, |p| {
            p.screen().contents().contains("cat completion_unique_file")
        });
        pane.preview_output(b"\x1b[2J\x1b[H");
        pane.key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        until(&mut pane, |p| {
            p.screen().contents().contains("HEART_FIXTURE>")
        });
        pane.send(b"sleep 30\r".to_vec());
        let pid = pane.running.as_ref().unwrap().pid.unwrap() as i32;
        until(&mut pane, |p| {
            p.running
                .as_ref()
                .unwrap()
                .master
                .process_group_leader()
                .is_some_and(|g| g != pid)
        });
        pane.key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        until(&mut pane, |p| {
            p.running.as_ref().unwrap().master.process_group_leader() == Some(pid)
        });
        pane.send(b"printf INTERRUPT_%s OK\r".to_vec());
        until(&mut pane, |p| {
            p.screen().contents().contains("INTERRUPT_OK")
        });
        pane.resize(9, 57);
        pane.send(b"stty size\r".to_vec());
        until(&mut pane, |p| p.screen().contents().contains("9 57"));
        assert_eq!(pane.screen().size(), (9, 57));
        pane.send(b"exit\r".to_vec());
        until(&mut pane, TerminalPane::ended);
        assert!(pane.status.contains("EXITED") || pane.status.contains("CLOSED"));
    }

    #[cfg(unix)]
    #[test]
    fn real_pty_paste_is_atomic_and_reports_round_trip() {
        let fixture = Fixture::new();
        let mut pane = fixture.pane();
        // zsh enables bracketed paste at the prompt. Newlines remain editing data.
        until(&mut pane, |p| p.screen().bracketed_paste());
        pane.paste("printf PASTE_%s OK\n");
        thread::sleep(Duration::from_millis(50));
        pane.drain();
        assert!(!pane.screen().contents().contains("PASTE_OK"));
        pane.key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        until(&mut pane, |p| p.screen().contents().contains("PASTE_OK"));
        // A raw foreground reader asks the parser for cursor status; the reply must
        // go to the PTY, never to the outer terminal or assistant.
        pane.send(b"stty -echo -icanon min 1 time 0; printf '\\033[6n'; dd bs=1 count=6 2>/dev/null | od -An -tx1; stty sane\r".to_vec());
        until(&mut pane, |p| p.screen().contents().contains("1b 5b"));
    }

    #[cfg(unix)]
    #[test]
    fn real_pty_output_and_input_flood_stay_bounded_and_cleanup_foreground_group() {
        use nix::unistd::{Pid, getpgrp, getsid};
        let fixture = Fixture::new();
        let mut pane = fixture.pane();
        let pid = pane.running.as_ref().unwrap().pid.unwrap() as i32;
        assert_eq!(getsid(Some(Pid::from_raw(pid))).unwrap().as_raw(), pid);
        assert_ne!(getpgrp().as_raw(), pid);
        assert!(!owned_group(getpgrp().as_raw(), pid));
        pane.send(b"yes FLOOD\r".to_vec());
        until(&mut pane, |p| p.screen().contents().contains("FLOOD\n"));
        let foreground = pane
            .running
            .as_ref()
            .unwrap()
            .master
            .process_group_leader()
            .unwrap();
        assert_ne!(foreground, pid);
        // Let reader queue fill; UI drain remains bounded and a stopped consumer
        // never makes the UI perform a blocking write.
        thread::sleep(Duration::from_millis(60));
        let start = Instant::now();
        for _ in 0..40 {
            pane.paste(&"x".repeat(INPUT_LIMIT));
        }
        assert!(pane.notice.is_some());
        pane.drain();
        assert!(start.elapsed() < Duration::from_secs(1));
        let start = Instant::now();
        drop(pane);
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(
            getsid(Some(Pid::from_raw(pid))).is_err(),
            "shell not reaped"
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        while owned_group(foreground, pid) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        // Linux can briefly retain an orphaned zombie; no process may still run.
        if owned_group(foreground, pid) {
            #[cfg(target_os = "linux")]
            assert_eq!(
                std::fs::read_to_string(format!("/proc/{foreground}/stat"))
                    .unwrap()
                    .split_whitespace()
                    .nth(2),
                Some("Z")
            );
            #[cfg(not(target_os = "linux"))]
            panic!("foreground process group survived cleanup");
        }
    }

    #[cfg(unix)]
    #[test]
    fn real_pty_shutdown_never_submits_pending_input_even_when_shell_ignores_hup_term() {
        use nix::unistd::{Pid, getsid};
        let fixture = Fixture::new();
        let mut pane = fixture.pane();
        let shell = Pid::from_raw(pane.running.as_ref().unwrap().pid.unwrap() as i32);
        pane.send(b"trap '' HUP TERM; printf ready > shutdown-ready.txt\r".to_vec());
        until(&mut pane, |_| {
            fixture.dir.join("shutdown-ready.txt").exists()
        });
        // This command deliberately has NO Enter. Waiting for its displayed echo
        // establishes that it reached the shell's editing buffer before closing.
        let marker = fixture.dir.join("shutdown-executed.txt");
        pane.send(b"printf unexpected > shutdown-executed.txt".to_vec());
        until(&mut pane, |p| {
            p.screen()
                .contents()
                .contains("printf unexpected > shutdown-executed.txt")
        });
        assert!(!marker.exists());
        let start = Instant::now();
        drop(pane);
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(
            getsid(Some(shell)).is_err(),
            "shell must be reaped before checking the marker"
        );
        assert!(
            !marker.exists(),
            "shutdown executed unsubmitted shell input"
        );
    }

    #[cfg(unix)]
    #[test]
    fn real_pty_cleanup_kills_hup_term_ignoring_pipeline_after_group_leader_exits() {
        use nix::{
            sys::signal::{Signal, kill},
            unistd::{Pid, getpgid, getpgrp, getsid},
        };
        // If an assertion fails, remove only the test process whose session we
        // have independently verified. Never leave the reproducer running.
        struct TestChild {
            pid: Pid,
            session: Pid,
        }
        impl Drop for TestChild {
            fn drop(&mut self) {
                if getsid(Some(self.pid)) == Ok(self.session) {
                    let _ = kill(self.pid, Signal::SIGKILL);
                }
            }
        }
        let fixture = Fixture::new();
        let mut pane = fixture.pane();
        let session = Pid::from_raw(pane.running.as_ref().unwrap().pid.unwrap() as i32);
        pane.send(
            b"true | sh -c 'trap \"\" HUP TERM; echo $$ > pipeline.pid; exec sleep 30'\r".to_vec(),
        );
        let pid_file = fixture.dir.join("pipeline.pid");
        until(&mut pane, |_| {
            std::fs::read_to_string(&pid_file).is_ok_and(|s| s.trim().parse::<i32>().is_ok())
        });
        let child = TestChild {
            pid: Pid::from_raw(
                std::fs::read_to_string(pid_file)
                    .unwrap()
                    .trim()
                    .parse()
                    .unwrap(),
            ),
            session,
        };
        assert_eq!(getsid(Some(child.pid)).unwrap(), session);
        let group = getpgid(Some(child.pid)).unwrap().as_raw();
        assert_ne!(
            group,
            child.pid.as_raw(),
            "fixture must have an exited pipeline leader"
        );
        until(&mut pane, |_| getsid(Some(Pid::from_raw(group))).is_err());
        let master = pane.running.as_ref().unwrap().master.as_ref();
        assert!(attached_foreground(master, group));
        for excluded in [0, 1, -1, getpgrp().as_raw(), session.as_raw()] {
            assert!(!attached_foreground(master, excluded));
        }
        assert!(
            !owned_group(group, session.as_raw()),
            "reproduce old leader-PID check failure"
        );
        let start = Instant::now();
        drop(pane);
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(getsid(Some(session)).is_err(), "shell not reaped");
        assert!(
            getsid(Some(child.pid)).is_err(),
            "pipeline member survived or was not reaped by its shell"
        );
    }
}
