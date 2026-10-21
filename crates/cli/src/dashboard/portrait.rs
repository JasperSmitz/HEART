//! One bundled image, one cached Kitty virtual placement; no stdin probing.
use super::theme::Theme;
use image::{
    DynamicImage, ImageFormat, RgbaImage,
    imageops::{self, FilterType},
};
use ratatui::{
    Frame,
    layout::{Rect, Size},
    style::Style,
    widgets::{Block, Paragraph, Wrap},
};
use ratatui_image::{
    FontSize, Image,
    protocol::{Protocol, kitty::Kitty},
};
use std::io::{self, Write};

pub const PASTEL_PNG: &[u8] = include_bytes!("../../assets/pastel.png");
// Reuse one ID on resize, rather than retaining unbounded terminal-side image data.
const IMAGE_ID: u32 = 0x4845_4152;

#[derive(Default)]
pub(super) struct Portrait {
    image: Option<DynamicImage>,
    font: Option<FontSize>,
    cached: Option<(Size, Protocol)>,
    interactive: bool,
}

fn known_terminal(
    term: &str,
    program: &str,
    multiplexer: bool,
    forced: bool,
    no_color: bool,
) -> bool {
    !forced
        && !no_color
        && !multiplexer
        && !term.starts_with("screen")
        && !term.starts_with("tmux")
        && (matches!(program, "ghostty" | "kitty")
            || matches!(term, "xterm-ghostty" | "xterm-kitty"))
}

fn font_size(size: crossterm::terminal::WindowSize) -> Option<FontSize> {
    let width = size.width.checked_div(size.columns)?;
    let height = size.height.checked_div(size.rows)?;
    // Reject unavailable or implausible ioctl metrics before allocating a pixel canvas.
    (width > 0 && width <= 128 && height > 0 && height <= 256)
        .then_some(FontSize::new(width, height))
}

/// Pixel-space contain bounds, shared by terminal encoding and simulated SVG composition.
pub fn contain_pixels(bounds: (u32, u32), source: (u32, u32)) -> (u32, u32, u32, u32) {
    let (bw, bh) = bounds;
    let (sw, sh) = source;
    if bw == 0 || bh == 0 || sw == 0 || sh == 0 {
        return (0, 0, 0, 0);
    }
    let (w, h) = if u64::from(bw) * u64::from(sh) <= u64::from(bh) * u64::from(sw) {
        (
            bw,
            (u64::from(bw) * u64::from(sh) / u64::from(sw)).max(1) as u32,
        )
    } else {
        (
            (u64::from(bh) * u64::from(sw) / u64::from(sh)).max(1) as u32,
            bh,
        )
    };
    ((bw - w) / 2, (bh - h) / 2, w, h)
}

impl Portrait {
    pub fn interactive() -> Self {
        let enabled = known_terminal(
            &std::env::var("TERM").unwrap_or_default(),
            &std::env::var("TERM_PROGRAM").unwrap_or_default(),
            std::env::var_os("TMUX").is_some() || std::env::var_os("STY").is_some(),
            std::env::var("HEART_PORTRAIT").is_ok_and(|v| v == "text"),
            // Kitty encodes image IDs in RGB text colors; honor Crossterm's NO_COLOR rule.
            std::env::var("NO_COLOR").is_ok_and(|v| !v.is_empty()),
        );
        if !enabled {
            return Self::default();
        }
        let font = crossterm::terminal::window_size().ok().and_then(font_size);
        Self {
            image: image::load_from_memory_with_format(PASTEL_PNG, ImageFormat::Png).ok(),
            font,
            interactive: true,
            cached: None,
        }
    }

    pub fn refresh_metrics(&mut self) {
        if !self.interactive {
            return;
        }
        let font = crossterm::terminal::window_size().ok().and_then(font_size);
        if self.font.map(|f| (f.width, f.height)) != font.map(|f| (f.width, f.height)) {
            self.font = font;
            self.cached = None;
        }
    }

    #[cfg(test)]
    pub(super) fn fixture() -> Self {
        Self {
            image: Some(image::load_from_memory_with_format(PASTEL_PNG, ImageFormat::Png).unwrap()),
            font: Some(FontSize::new(9, 18)),
            cached: None,
            interactive: false,
        }
    }

    fn encode(&mut self, size: Size) -> Option<&Protocol> {
        let font = self.font?;
        let source = self.image.as_ref()?;
        if self.cached.as_ref().is_none_or(|(s, _)| *s != size) {
            let width = u32::from(size.width) * u32::from(font.width);
            let height = u32::from(size.height) * u32::from(font.height);
            let (x, y, w, h) = contain_pixels((width, height), (source.width(), source.height()));
            if w == 0 || h == 0 {
                return None;
            }
            let scaled = source.resize_exact(w, h, FilterType::Nearest);
            let mut canvas = RgbaImage::new(width, height);
            imageops::overlay(&mut canvas, &scaled, i64::from(x), i64::from(y));
            let kitty = Kitty::new(canvas.into(), size, IMAGE_ID, false, false).ok()?;
            self.cached = Some((size, Protocol::Kitty(kitty)));
        }
        self.cached.as_ref().map(|(_, p)| p)
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if theme.portrait == "cat"
            && let Some(protocol) = self.encode(area.into())
        {
            frame.render_widget(Image::new(protocol), area);
            return;
        }
        frame.render_widget(
            Paragraph::new(if area.height < 8 {
                "Portrait unavailable\nText mode".into()
            } else {
                format!("{}\n\nPortrait unavailable\nText mode", theme.name)
            })
            .wrap(Wrap { trim: false })
            .centered()
            .style(Style::default().fg(theme.gold))
            .block(if area.height < 8 {
                Block::default()
            } else {
                Block::bordered().border_style(Style::default().fg(theme.muted))
            }),
            area,
        );
    }
}

impl Drop for Portrait {
    fn drop(&mut self) {
        if self.interactive {
            // Uppercase I frees only our image and its placements. No terminal query/reply.
            let mut out = io::stdout();
            let _ = write!(out, "\x1b_Ga=d,d=I,i={IMAGE_ID},q=2;\x1b\\");
            let _ = out.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contain_keeps_proportions_and_centers_in_pixel_space() {
        assert_eq!(contain_pixels((252, 216), (1254, 1254)), (18, 0, 216, 216));
        assert_eq!(contain_pixels((90, 180), (1254, 1254)), (0, 45, 90, 90));
        assert_eq!(contain_pixels((200, 100), (400, 200)), (0, 0, 200, 100));
        assert_eq!(contain_pixels((0, 10), (1, 1)), (0, 0, 0, 0));
    }
    #[test]
    fn conservative_detection_and_missing_metrics_use_text() {
        assert!(known_terminal(
            "xterm-ghostty",
            "ghostty",
            false,
            false,
            false
        ));
        assert!(known_terminal("xterm-kitty", "", false, false, false));
        assert!(!known_terminal(
            "xterm-ghostty",
            "ghostty",
            false,
            false,
            true
        ));
        for (term, program, mux, forced) in [
            ("xterm", "", false, false),
            ("screen", "ghostty", false, false),
            ("xterm-ghostty", "ghostty", true, false),
            ("xterm-ghostty", "ghostty", false, true),
        ] {
            assert!(!known_terminal(term, program, mux, forced, false));
        }
        assert!(
            font_size(crossterm::terminal::WindowSize {
                rows: 24,
                columns: 80,
                width: 0,
                height: 0
            })
            .is_none()
        );
        assert!(
            font_size(crossterm::terminal::WindowSize {
                rows: 0,
                columns: 0,
                width: 800,
                height: 480
            })
            .is_none()
        );
        let f = font_size(crossterm::terminal::WindowSize {
            rows: 24,
            columns: 80,
            width: 800,
            height: 480,
        })
        .unwrap();
        assert_eq!((f.width, f.height), (10, 20));
    }
    #[test]
    fn decode_once_cache_protocol_until_size_changes_and_fail_closed() {
        let mut p = Portrait::fixture();
        assert_eq!(p.image.as_ref().unwrap().width(), 1254);
        let size = Size::new(24, 12);
        use ratatui::{buffer::Buffer, widgets::Widget};
        let area = Rect::new(0, 0, size.width, size.height);
        let mut buffer = Buffer::empty(area);
        Image::new(p.encode(size).unwrap()).render(area, &mut buffer);
        assert!(buffer[(0, 0)].symbol().contains("\x1b_G"));
        Image::new(p.encode(size).unwrap()).render(area, &mut buffer);
        // A newly encoded protocol would retransmit. Idle draws reuse the transmitted state.
        assert!(!buffer[(0, 0)].symbol().contains("\x1b_G"));
        assert_eq!(
            p.encode(Size::new(20, 10)).unwrap().size(),
            Size::new(20, 10)
        );
        p.image = None;
        assert!(p.encode(size).is_none());
        p.font = None;
        assert!(p.encode(size).is_none());
    }
}
