//! Deterministic styled-cell export using the production renderer. No engine/config/services.
use heart::{
    dashboard::{self, Dashboard},
    session::Session,
};
use heart_core::Personality;
use ratatui::{Terminal, backend::TestBackend, style::Color};
use std::{collections::BTreeMap, fmt::Write, io};
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn color(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        _ => "#ffe9d4".into(),
    }
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
    app.input = "/disk /".into();
    if mode == "help" {
        app.help = true;
    }
    if mode == "long" {
        app.dialogue.extend((1..12).map(|i| format!("{} · Source [{i}]: A quiet corner for ideas, little questions and useful answers. https://example.com/evidence/{i}",selected)));
    }
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| dashboard::draw(frame, &mut app))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\"><rect width=\"100%\" height=\"100%\" fill=\"#201923\"/>",
        width * 9,
        height * 18,
        width * 9,
        height * 18
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
                write!(svg,"<text x=\"{}\" y=\"{}\" font-family=\"Menlo,monospace\" font-size=\"14\" fill=\"{}\">{}</text>",x*9,y*18+14,color(cell.fg),escape(cell.symbol())).unwrap();
            }
        }
    }
    svg.push_str("</svg>");
    println!("{svg}");
    Ok(())
}
