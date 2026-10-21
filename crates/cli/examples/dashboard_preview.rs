//! Deterministic styled-cell export using the production renderer. No engine/config/services.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use heart::{
    dashboard::{self, Dashboard},
    session::Session,
};
use heart_core::Personality;
use ratatui::{
    Terminal,
    backend::TestBackend,
    style::{Color, Modifier},
};
use std::{collections::BTreeMap, fmt::Write, io};
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn color(c: Color) -> String {
    let index = match c {
        Color::Rgb(r, g, b) => return format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Indexed(n) => n,
        Color::Black => 0,
        Color::Red => 1,
        Color::Green => 2,
        Color::Yellow => 3,
        Color::Blue => 4,
        Color::Magenta => 5,
        Color::Cyan => 6,
        Color::Gray => 7,
        Color::DarkGray => 8,
        Color::LightRed => 9,
        Color::LightGreen => 10,
        Color::LightYellow => 11,
        Color::LightBlue => 12,
        Color::LightMagenta => 13,
        Color::LightCyan => 14,
        Color::White => 15,
        Color::Reset => return "#ffe9d4".into(),
    };
    if index < 16 {
        return [
            "#000000", "#800000", "#008000", "#808000", "#000080", "#800080", "#008080", "#c0c0c0",
            "#808080", "#ff0000", "#00ff00", "#ffff00", "#0000ff", "#ff00ff", "#00ffff", "#ffffff",
        ][usize::from(index)]
        .into();
    }
    let (r, g, b) = if index >= 232 {
        let gray = 8 + (index - 232) * 10;
        (gray, gray, gray)
    } else {
        let n = index - 16;
        let level = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
        (level(n / 36), level(n / 6 % 6), level(n % 6))
    };
    format!("#{r:02x}{g:02x}{b:02x}")
}
fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let selected = args.get(1).map(String::as_str).unwrap_or("cat");
    let width = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(120);
    let height = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(40);
    let mode = args.get(4).map(String::as_str).unwrap_or("home");
    let profiles = BTreeMap::from([
        ("cat".into(), Personality::cat()),
        ("butler".into(), Personality::butler()),
    ]);
    if !profiles.contains_key(selected) {
        return Err(io::Error::other("Expected cat or butler"));
    }
    let session = Session {
        profiles,
        selected: selected.into(),
        help:
            "/disk [path] · /uptime · /system · /search <query>\n/profile ID · /ID · /help · /exit"
                .into(),
        experimental_nl: false,
        diagnostics: false,
    };
    let mut app = Dashboard::new(session, false);
    app.preview_terminal(width, height, b"\x1b[1;35mjasper\x1b[0m in \x1b[36m~/HEART\x1b[0m\r\n\x1b[32m>\x1b[0m cargo test --workspace\r\n\x1b[32mtest result: ok\x1b[0m\r\n\r\n\x1b[1;35mjasper\x1b[0m in \x1b[36m~/HEART\x1b[0m\r\n\x1b[32m>\x1b[0m ");
    app.input = "/disk /".into();
    if mode.contains("help") {
        app.help = true;
    }
    if mode.contains("long") {
        app.focus = dashboard::Focus::Dialogue;
        app.dialogue.extend((1..12).map(|i| format!("{} · Source [{i}]: A quiet corner for ideas, little questions and useful answers. https://example.com/evidence/{i}",if selected == "cat" { "Pastel" } else { "Butler" })));
    }
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| dashboard::draw(frame, &mut app))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\"><rect width=\"100%\" height=\"100%\" fill=\"#201923\"/>",
        width * 9,
        height * 18 + 18,
        width * 9,
        height * 18 + 18
    );
    for y in 0..height {
        for x in 0..width {
            let cell = &buffer[(x, y)];
            if let Color::Rgb(..) = cell.bg {
                write!(
                    svg,
                    "<rect x=\"{}\" y=\"{}\" width=\"9\" height=\"18\" fill=\"{}\"/>",
                    x * 9,
                    y * 18,
                    color(cell.bg)
                )
                .unwrap();
            }
            if cell.symbol().trim().is_empty() {
                continue;
            }
            // Block glyphs are exported as rectangles to preserve their cell geometry.
            let block = match cell.symbol() {
                "█" => Some((0, 18)),
                "▄" => Some((9, 9)),
                "▀" => Some((0, 9)),
                _ => None,
            };
            if let Some((offset, h)) = block {
                write!(
                    svg,
                    "<rect x=\"{}\" y=\"{}\" width=\"9\" height=\"{h}\" fill=\"{}\"/>",
                    x * 9,
                    y * 18 + offset,
                    color(cell.fg)
                )
                .unwrap();
            } else {
                write!(svg,"<text x=\"{}\" y=\"{}\" font-family=\"Menlo,monospace\" font-size=\"14\" font-weight=\"{}\" fill=\"{}\">{}</text>",x*9,y*18+14,if cell.modifier.contains(Modifier::BOLD) { "bold" } else { "normal" },color(cell.fg),escape(cell.symbol())).unwrap();
            }
        }
    }
    let simulated_rect = app
        .portrait_rect
        .filter(|_| mode.starts_with("image") && selected == "cat");
    if let Some(rect) = simulated_rect {
        // Exact bundled bytes, same centered pixel-space contain as production encoding.
        let (x, y, w, h) = dashboard::contain_pixels(
            (u32::from(rect.width) * 9, u32::from(rect.height) * 18),
            (1254, 1254),
        );
        let background = color(buffer[(rect.x, rect.y)].bg);
        write!(svg, "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{background}\"/><image x=\"{}\" y=\"{}\" width=\"{w}\" height=\"{h}\" preserveAspectRatio=\"xMidYMid meet\" style=\"image-rendering:pixelated\" href=\"data:image/png;base64,{}\"/>",
            rect.x * 9, rect.y * 18, rect.width * 9, rect.height * 18,
            u32::from(rect.x) * 9 + x, u32::from(rect.y) * 18 + y,
            STANDARD.encode(dashboard::PASTEL_PNG)).unwrap();
    }
    // Clearly distinguish a composed preview from a real emulator screenshot.
    write!(svg, "<rect x=\"0\" y=\"{}\" width=\"100%\" height=\"18\" fill=\"#201923\"/><text x=\"4\" y=\"{}\" font-family=\"monospace\" font-size=\"11\" fill=\"#f6c9d2\">{}</text>",
        height * 18, height * 18 + 13,
        if simulated_rect.is_some() { "SIMULATED GRAPHICS · exact bundled PNG · 9×18 cells" } else { "STYLED CELL PREVIEW · text mode · 9×18 cells" }).unwrap();
    svg.push_str("</svg>");
    println!("{svg}");
    Ok(())
}
