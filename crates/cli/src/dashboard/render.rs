use super::{Dashboard, Focus, theme::Theme};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Clear, Padding, Paragraph, Wrap},
};
use tui_term::widget::PseudoTerminal;

/// One geometry calculation drives the renderer AND kernel/parser dimensions.
pub(super) struct Areas {
    pub header: Rect,
    pub shell: Rect,
    pub shell_inner: Rect,
    pub dialogue: Rect,
    pub input: Rect,
    pub footer: Rect,
    pub system: Rect,
    pub music: Rect,
}
pub(super) fn areas(size: Rect) -> Areas {
    let header = if size.height < 3 {
        0
    } else if size.height >= 32 && size.width >= 100 {
        3
    } else {
        1
    };
    let footer = u16::from(size.height >= 3);
    let content = Rect::new(
        size.x,
        size.y + header,
        size.width,
        size.height - header - footer,
    );
    let input_height = if size.height >= 10 {
        3
    } else {
        u16::from(size.height >= 4)
    };
    // Side-by-side retains at least 65 shell columns; narrower/shorter screens
    // use a small bottom companion and input instead of squeezing the shell.
    let sidebar_width = if size.width >= 100 && size.height >= 16 {
        34
    } else {
        0
    };
    let (shell, dialogue, input, system, music) = if sidebar_width > 0 {
        let right = Rect::new(
            content.right() - sidebar_width,
            content.y,
            sidebar_width,
            content.height,
        );
        let utility_height = if wide(size.width, size.height) { 5 } else { 0 };
        let system = Rect::new(right.x, right.y, right.width, utility_height);
        let music = Rect::new(right.x, system.bottom(), right.width, utility_height);
        let input = Rect::new(
            right.x,
            right.bottom() - input_height,
            right.width,
            input_height,
        );
        let dialogue = Rect::new(
            right.x,
            music.bottom(),
            right.width,
            input.y - music.bottom(),
        );
        (
            Rect::new(
                content.x,
                content.y,
                content.width - sidebar_width - 1,
                content.height,
            ),
            dialogue,
            input,
            system,
            music,
        )
    } else {
        let companion = if size.height >= 20 {
            6
        } else if size.height >= 13 {
            3
        } else {
            0
        };
        let companion = companion.min(content.height.saturating_sub(input_height + 3));
        let shell = Rect::new(
            content.x,
            content.y,
            content.width,
            content.height - input_height - companion,
        );
        let dialogue = Rect::new(content.x, shell.bottom(), content.width, companion);
        let input = Rect::new(content.x, dialogue.bottom(), content.width, input_height);
        (shell, dialogue, input, Rect::default(), Rect::default())
    };
    let shell_inner = if shell.width >= 3 && shell.height >= 3 {
        Rect::new(shell.x + 1, shell.y + 1, shell.width - 2, shell.height - 2)
    } else {
        shell
    };
    Areas {
        header: Rect::new(size.x, size.y, size.width, header),
        shell,
        shell_inner,
        dialogue,
        input,
        footer: Rect::new(
            size.x,
            size.bottom().saturating_sub(footer),
            size.width,
            footer,
        ),
        system,
        music,
    }
}
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
    text(
        frame,
        inner,
        vec![
            Line::styled(
                "CPU   32%       42 °C",
                Style::default().fg(theme.meters[0]),
            ),
            Line::styled("Memory   9.3 / 16 GB", Style::default().fg(theme.meters[1])),
            Line::styled("Disk   322 / 476 GB", Style::default().fg(theme.meters[2])),
        ],
        Style::default().fg(theme.text),
    );
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
    text(
        frame,
        inner,
        vec![
            Line::from(format!("{track} · {artist}")),
            Line::styled(
                format!(
                    "◁  {}  ▷   1:24 / 3:12",
                    if app.playing { "Ⅱ" } else { "▶" }
                ),
                Style::default().fg(theme.gold),
            ),
            Line::from("Space/Enter · ←/→ track"),
        ],
        Style::default().fg(theme.text),
    );
}

fn dialogue(frame: &mut Frame, area: Rect, app: &mut Dashboard, theme: &Theme) {
    let panel = Block::bordered()
        .border_type(BorderType::Double)
        .title(format!(" DIALOGUE · {} ", theme.name))
        .border_style(Style::default().fg(if app.focus == Focus::Dialogue {
            theme.accent
        } else {
            theme.gold
        }))
        .style(Style::default().bg(theme.surface).fg(theme.text))
        .padding(if area.width >= 55 && area.height >= 10 {
            Padding::new(2, 2, 1, 1)
        } else {
            Padding::ZERO
        });
    let inner = panel.inner(area);
    frame.render_widget(panel, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }
    // Sidebar portraits sit above the text; wider fallback dialogue keeps them
    // alongside it. Both leave room for the latest response and history controls.
    let stacked = inner.width < 48;
    let portrait_width = if stacked && inner.width >= 26 && inner.height >= 10 {
        16
    } else if !stacked && inner.height >= 4 {
        (inner.width / 4).min(20)
    } else {
        0
    };
    let portrait_height = if stacked { 4 } else { inner.height.min(6) };
    if portrait_width > 0 && !app.help {
        let portrait_area = Rect::new(
            inner.x
                + if stacked {
                    (inner.width - portrait_width) / 2
                } else {
                    0
                },
            inner.y,
            portrait_width,
            portrait_height,
        );
        app.portrait_rect = Some(portrait_area);
        app.portrait.draw(frame, portrait_area, theme);
    }
    let gap = if portrait_width > 0 && !stacked {
        portrait_width + 3
    } else {
        0
    };
    let top = if portrait_width > 0 && stacked {
        portrait_height
    } else {
        0
    };
    let body = Rect::new(
        inner.x + gap,
        inner.y + top,
        inner.width - gap,
        inner.height - top,
    );
    let rows = Layout::vertical([
        Constraint::Length(u16::from(body.height >= 2)),
        Constraint::Min(0),
        Constraint::Length(if body.height >= 5 { 1 } else { 0 }),
    ])
    .split(body);
    text(
        frame,
        rows[0],
        Line::from(vec![
            Span::styled(
                theme.name.clone(),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if app.busy {
                    "  ·  working…"
                } else if portrait_width == 0 {
                    "  ·  text mode"
                } else {
                    "  ·  here with you"
                },
                Style::default().fg(theme.gold),
            ),
        ]),
        Style::default(),
    );
    let current = [app.companion.clone()];
    let lines: Vec<_> = if app.focus == Focus::Dialogue {
        app.dialogue.as_slice()
    } else {
        &current
    }
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
        .line_count(rows[1].width)
        .saturating_sub(usize::from(rows[1].height))
        .min(usize::from(u16::MAX)) as u16;
    app.scroll = app.scroll.min(max);
    frame.render_widget(
        p.scroll((
            if app.focus == Focus::Dialogue {
                app.scroll
            } else {
                0
            },
            0,
        )),
        rows[1],
    );
    let continuation = if app.scroll < max {
        "↓ more · Tab to dialogue · PgDn"
    } else if max > 0 {
        "↑ earlier · Tab to dialogue · PgUp"
    } else {
        "◆"
    };
    frame.render_widget(
        Paragraph::new(continuation)
            .right_aligned()
            .style(Style::default().fg(theme.gold)),
        rows[2],
    );
}
fn input(frame: &mut Frame, area: Rect, app: &Dashboard, theme: &Theme) {
    let panel = block(
        if app.busy {
            " HEART INPUT · working; draft preserved "
        } else {
            " HEART INPUT · Enter to send "
        },
        theme,
        app.focus == Focus::Input,
    );
    let inner = if area.height >= 3 {
        panel.inner(area)
    } else {
        area
    };
    if area.height >= 3 {
        frame.render_widget(panel, area);
    }
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
    app.portrait_rect = None;
    app.normalize_focus(size.width, size.height);
    let theme = Theme::for_profile(app.session.profile());
    frame.render_widget(
        Block::default().style(Style::default().bg(theme.background).fg(theme.text)),
        size,
    );
    let areas = areas(size);
    app.shell
        .resize(areas.shell_inner.height, areas.shell_inner.width);
    let status = if app.busy { "WORKING" } else { "READY" };
    let title = Line::from(vec![
        Span::styled(
            " HEART ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            "· {} · {status} · {}",
            theme.name,
            if app.focus == Focus::Shell {
                "SHELL"
            } else {
                "HEART"
            }
        )),
        Span::styled(
            "    a little space for your day",
            Style::default().fg(theme.gold),
        ),
    ]);
    let p = Paragraph::new(title).style(Style::default().fg(theme.text));
    frame.render_widget(
        if areas.header.height == 1 {
            p
        } else {
            p.block(block(" ♡ ", &theme, false))
        },
        areas.header,
    );
    if areas.system.height > 0 {
        system(frame, areas.system, &theme);
    }
    if areas.music.height > 0 {
        music(frame, areas.music, app, &theme);
    }
    let focused = app.focus == Focus::Shell;
    let title = format!(
        " SHELL · {} · {} ",
        if focused { "FOCUSED" } else { "F12 to focus" },
        app.shell.label()
    );
    if areas.shell_inner != areas.shell {
        frame.render_widget(block(title, &theme, focused), areas.shell);
    }
    frame.render_widget(
        PseudoTerminal::new(app.shell.screen())
            .cursor(tui_term::widget::Cursor::default().visibility(false)),
        areas.shell_inner,
    );
    if focused
        && !app.help
        && !app.shell.screen().hide_cursor()
        && app.shell.screen().scrollback() == 0
    {
        let (row, col) = app.shell.screen().cursor_position();
        if row < areas.shell_inner.height && col < areas.shell_inner.width {
            frame.set_cursor_position((areas.shell_inner.x + col, areas.shell_inner.y + row));
        }
    }
    if areas.dialogue.height > 0 {
        dialogue(frame, areas.dialogue, app, &theme);
    }
    input(frame, areas.input, app, &theme);
    let footer = if let Some(notice) = &app.shell.notice {
        notice.as_str()
    } else if focused {
        if size.width >= 75 {
            " SHELL · F12 HEART input · Shift-PgUp/PgDn scrollback · typing returns live"
        } else {
            "SHELL · F12 HEART · Shift-PgUp/PgDn scroll"
        }
    } else if size.width >= 75 {
        " HEART · F12 shell · Tab focus · F1 help · F2 character · Ctrl-Q quit"
    } else {
        "HEART · F12 shell · F1 help · F2 profile · ^Q quit"
    };
    text(
        frame,
        areas.footer,
        footer.to_owned(),
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
            "{}\n\nF12: toggle shell / HEART input (global)\nShell: all ordinary keys go to the child, including Ctrl-C/Q, Tab, F1/F2.\nShift-PgUp/PgDn: bounded shell scrollback; PgDn or typing returns live.\nHEART: Tab / Shift-Tab focus visible panels\nF2: next character/theme · F1 / Esc: close help\nInput: ←/→ Home/End edit · ↑/↓ history\nDialogue: ↑/↓ PgUp/PgDn Home/End scroll\nMusic: Enter/Space play · ←/→ track (mock only)\nHEART only: Ctrl-Q / Ctrl-C / /exit quit\n\nWidgets: STATIC DEMO / LOCAL MOCK. HEART commands use real modules.\nShell is manual only; assistant has no shell access. READY means UI ready.",
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
