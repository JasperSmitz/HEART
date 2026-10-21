use super::{Dashboard, Focus, art, theme::Theme};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Clear, Gauge, Paragraph, Wrap},
};
pub(super) fn wide(width: u16, height: u16) -> bool {
    width >= 100 && height >= 30
}
fn block<'a>(title: impl Into<Line<'a>>, theme: &Theme, focused: bool) -> Block<'a> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .title(title)
        .border_style(Style::default().fg(if focused { theme.gold } else { theme.accent }))
        .style(Style::default().bg(theme.surface).fg(theme.text))
}
fn text(frame: &mut Frame, area: Rect, value: impl Into<Text<'static>>, style: Style) {
    frame.render_widget(Paragraph::new(value).style(style), area);
}
fn system(frame: &mut Frame, area: Rect, theme: &Theme) {
    let panel = block(" SYSTEM · STATIC DEMO ", theme, false);
    let inner = panel.inner(area);
    frame.render_widget(panel, area);
    if inner.height < 9 {
        return;
    }
    for (i, (label, percent)) in [
        ("CPU   32%", 32),
        ("Memory   9.3 / 16 GB", 58),
        ("Disk   322 / 476 GB", 68),
    ]
    .into_iter()
    .enumerate()
    {
        let y = inner.y + i as u16 * 3;
        text(
            frame,
            Rect::new(inner.x + 1, y, inner.width.saturating_sub(2), 1),
            label,
            Style::default().fg(theme.text),
        );
        frame.render_widget(
            Gauge::default()
                .ratio(percent as f64 / 100.0)
                .label("")
                .gauge_style(Style::default().fg(theme.accent).bg(theme.background)),
            Rect::new(inner.x + 1, y + 1, inner.width.saturating_sub(3), 1),
        );
    }
    if inner.height > 10 {
        text(
            frame,
            Rect::new(inner.x + 1, inner.y + 10, inner.width.saturating_sub(2), 1),
            "Temperature   42 °C",
            Style::default().fg(theme.gold),
        );
    }
}
fn music(frame: &mut Frame, area: Rect, app: &Dashboard, theme: &Theme) {
    let panel = block(" MUSIC · LOCAL MOCK ", theme, app.focus == Focus::Music);
    let inner = panel.inner(area);
    frame.render_widget(panel, area);
    let tracks = [
        ("Moonlit Walk", "HEART room radio"),
        ("Rose Window", "The Paper Moons"),
        ("Tea at Midnight", "Quiet Company"),
    ];
    let (track, artist) = tracks[app.track];
    let rows = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Min(1),
    ])
    .split(inner);
    text(
        frame,
        rows[0],
        vec![
            Line::from("  ╭──────╮"),
            Line::from("  │  ◉   │  night radio"),
            Line::from("  ╰──────╯"),
        ],
        Style::default().fg(theme.accent),
    );
    text(
        frame,
        rows[1],
        vec![Line::from(track), Line::from(artist)],
        Style::default().fg(theme.text),
    );
    text(
        frame,
        rows[2],
        format!(
            "  ◁   {}   ▷    1:24 / 3:12",
            if app.playing { "Ⅱ" } else { "▶" }
        ),
        Style::default().fg(theme.gold),
    );
    text(
        frame,
        rows[3],
        "Enter/Space play · ←/→ track\nLocal state only · no audio",
        Style::default().fg(theme.accent),
    );
}
fn room(frame: &mut Frame, area: Rect, app: &Dashboard, theme: &Theme) {
    let panel = block(" HEART · a smaller day, a brighter you ", theme, false);
    let inner = panel.inner(area);
    frame.render_widget(panel, area);
    if inner.height == 0 {
        return;
    }
    let character_width = (inner.height.saturating_mul(2).saturating_mul(30) / 28)
        .min(inner.width.saturating_sub(2))
        .min(64);
    let character = Rect::new(
        inner.x + (inner.width - character_width) / 2,
        inner.y,
        character_width,
        inner.height,
    );
    let left_width = character.x.saturating_sub(inner.x).saturating_sub(2);
    let right_x = character.right().saturating_add(2);
    let right_width = inner.right().saturating_sub(right_x);
    if left_width >= 12 && inner.height >= 6 {
        let window = Rect::new(
            inner.x + 1,
            inner.y + 1,
            left_width.min(18),
            inner.height.saturating_sub(2).min(8),
        );
        let panel = Block::bordered()
            .title(" night ")
            .border_style(Style::default().fg(theme.accent))
            .style(Style::default().bg(theme.background));
        frame.render_widget(
            Paragraph::new("  ·   +   ☾\n     ·   *\n +      ·\n  ▄  ▄▄   ▄\n ▄█▄▄██▄▄▄█")
                .style(Style::default().fg(theme.gold))
                .block(panel),
            window,
        );
    }
    if right_width >= 13 && inner.height >= 9 {
        text(
            frame,
            Rect::new(right_x, inner.y + 1, 13, 5),
            "  *  .  *\n  GOOD IDEAS\n  BETTER DAYS\n  TOGETHER\n     ♡",
            Style::default().fg(theme.gold),
        );
        text(
            frame,
            Rect::new(right_x, inner.bottom().saturating_sub(4), 13, 4),
            "   ╭──╮\n   │♡ ├╮\n ▄▄╰──╯╯\n ▀▀▀▀▀▀▀▀",
            Style::default().fg(theme.accent),
        );
    }
    art::portrait(
        frame,
        character,
        theme,
        app.animations && app.tick % 12 == 11,
    );
    if left_width >= 12 && inner.height > 12 {
        text(
            frame,
            Rect::new(inner.x + 2, inner.bottom().saturating_sub(4), 12, 4),
            "   ╱╲\n  ╱❋ ╲\n   ▐▌\n  ▀▀▀▀",
            Style::default().fg(theme.gold),
        );
    }
}
fn dialogue(frame: &mut Frame, area: Rect, app: &mut Dashboard, theme: &Theme) {
    let panel = block(
        format!(
            " {} · DIALOGUE {} ",
            theme.name,
            if app.focus == Focus::Dialogue {
                "↑↓ PgUp/PgDn"
            } else {
                ""
            }
        ),
        theme,
        app.focus == Focus::Dialogue,
    );
    let inner = panel.inner(area);
    frame.render_widget(panel, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    let portrait_width = if inner.width >= 55 && inner.height >= 4 {
        13
    } else {
        0
    };
    if portrait_width > 0 {
        art::portrait(
            frame,
            Rect::new(inner.x, inner.y, portrait_width, inner.height.min(8)),
            theme,
            false,
        );
    }
    let body = Rect::new(
        inner.x + portrait_width,
        inner.y,
        inner.width.saturating_sub(portrait_width),
        inner.height,
    );
    let lines: Vec<_> = app
        .dialogue
        .iter()
        .flat_map(|line| {
            line.lines()
                .map(|l| Line::from(l.to_owned()))
                .chain(std::iter::once(Line::from("")))
        })
        .collect();
    let p = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(theme.text));
    let max = p
        .line_count(body.width)
        .saturating_sub(usize::from(body.height))
        .min(usize::from(u16::MAX)) as u16;
    app.scroll = app.scroll.min(max);
    frame.render_widget(p.scroll((app.scroll, 0)), body);
}
fn input(frame: &mut Frame, area: Rect, app: &Dashboard, theme: &Theme) {
    let panel = block(
        if app.busy {
            " MESSAGE · working; draft preserved "
        } else {
            " MESSAGE · Enter to send "
        },
        theme,
        app.focus == Focus::Input,
    );
    let inner = panel.inner(area);
    frame.render_widget(panel, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    let max = usize::from(inner.width.saturating_sub(3));
    // Ratatui measures display columns, including wide Unicode and zero-width combining marks.
    let mut start = app.cursor;
    let mut columns = 0;
    for (i, c) in app.input[..app.cursor].char_indices().rev() {
        let w = Span::raw(c.to_string()).width();
        if columns + w > max {
            break;
        }
        columns += w;
        start = i;
    }
    let content = if app.input.is_empty() {
        "Type a command… /help"
    } else {
        &app.input[start..]
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("› ", Style::default().fg(theme.gold)),
            Span::raw(content),
        ]))
        .style(Style::default().fg(theme.text)),
        inner,
    );
    if app.focus == Focus::Input && !app.help {
        frame.set_cursor_position((
            inner
                .x
                .saturating_add(2)
                .saturating_add(columns as u16)
                .min(inner.right().saturating_sub(1)),
            inner.y,
        ));
    }
}
pub fn draw(frame: &mut Frame, app: &mut Dashboard) {
    let size = frame.area();
    app.normalize_focus(size.width, size.height);
    let theme = Theme::for_profile(app.session.profile());
    frame.render_widget(
        Block::default().style(Style::default().bg(theme.background).fg(theme.text)),
        size,
    );
    let tiny = size.height < 12 || size.width < 38;
    let header = if tiny || size.width < 100 { 1 } else { 3 };
    let rows = Layout::vertical([
        Constraint::Length(header),
        Constraint::Min(0),
        Constraint::Length(if size.height >= 6 { 3 } else { 1 }),
        Constraint::Length(1),
    ])
    .split(size);
    let status = if app.busy { "WORKING" } else { "READY" };
    let title = Line::from(vec![
        Span::styled(
            " HEART ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("· {} · {status}", theme.name)),
        Span::styled(
            "    your cozy command room",
            Style::default().fg(theme.gold),
        ),
    ]);
    let p = Paragraph::new(title).style(Style::default().fg(theme.text));
    frame.render_widget(
        if header == 1 {
            p
        } else {
            p.block(block(" ♡ ", &theme, false))
        },
        rows[0],
    );
    let content = rows[1];
    let character_area = if wide(size.width, size.height) {
        let cols = Layout::horizontal([Constraint::Percentage(27), Constraint::Percentage(73)])
            .split(content);
        let widgets = Layout::vertical([Constraint::Percentage(53), Constraint::Percentage(47)])
            .split(cols[0]);
        system(frame, widgets[0], &theme);
        music(frame, widgets[1], app, &theme);
        cols[1]
    } else {
        content
    };
    if tiny {
        dialogue(frame, character_area, app, &theme);
    } else {
        let dialogue_height = if size.height >= 30 { 9 } else { 6 };
        let main = Layout::vertical([Constraint::Min(3), Constraint::Length(dialogue_height)])
            .split(character_area);
        room(frame, main[0], app, &theme);
        dialogue(frame, main[1], app, &theme);
    }
    input(frame, rows[2], app, &theme);
    text(
        frame,
        rows[3],
        " Tab focus · F1 help · F2 character · Ctrl-Q quit",
        Style::default().fg(theme.accent),
    );
    if app.help {
        let margin = if size.width > 60 { 8 } else { 0 };
        let area = Rect::new(
            size.x + margin,
            size.y + size.height.min(2),
            size.width.saturating_sub(margin * 2),
            size.height.saturating_sub(4),
        );
        frame.render_widget(Clear, area);
        let help = format!(
            "{}\n\nTab / Shift-Tab: focus visible panels\nF2: next character/theme · F1 / Esc: close help\nInput: ←/→ Home/End edit · ↑/↓ history\nDialogue: ↑/↓ PgUp/PgDn Home/End scroll\nMusic: Enter/Space play · ←/→ track (mock only)\nCtrl-Q / Ctrl-C / /exit: quit immediately\n\nWidgets: STATIC DEMO / LOCAL MOCK. Typed commands use real modules.\nREADY means UI ready, not provider connectivity.",
            app.session.help
        );
        let panel = block(" HELP · ↑↓ scroll · F1 / Esc ", &theme, true);
        let inner = panel.inner(area);
        let paragraph = Paragraph::new(help).wrap(Wrap { trim: false });
        let max = paragraph
            .line_count(inner.width)
            .saturating_sub(usize::from(inner.height))
            .min(usize::from(u16::MAX)) as u16;
        app.help_scroll = app.help_scroll.min(max);
        frame.render_widget(paragraph.scroll((app.help_scroll, 0)).block(panel), area);
    }
}
