//! Original cell-pixel portraits. Each source pixel occupies two terminal columns.
use super::theme::Theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
};
const CAT: &[&str] = &[
    "      rr              rr      ",
    "     rpph            hppr     ",
    "    rppphh          hhpppr    ",
    "   rpppphhhrrrrrrrrhhhppppr   ",
    "   rppphrrrrpppppprrrrhpppr   ",
    "    rhrrpppppppppppppprrhr   ",
    "     rrpppphwwppppppppprr    ",
    "    rrpppphwwwpppppppppprr   ",
    "   rrpppppppppppppppppppprr  ",
    "   rppppphpppppppphpppppppr  ",
    "  rrpppphsspppppphsspppppprr ",
    "  rpppphsssspppphssssppppppr ",
    "  rppphssssshpphssssshppppr ",
    "  rppphsKKssshhssKKsshppppr ",
    " rrppphswTssssssswTsshpppprr",
    " rpppphssTssssssssTsshpppppr",
    " rpppphsssbssssssbssshpppppr",
    " rrpppphssssmmmmsssshpppprr ",
    "  rppppphssssmmsssshpppppr  ",
    " rppppppphssssssshpppppppr  ",
    "rrpppppppphssssshpppppppprr ",
    "rppppppprrddggddrrppppppppr",
    " rppppprwwddggddwwrppppppr ",
    "  rppprwwwwddddwwwwrppppr  ",
    "   rrwwwwwwddddwwwwwwrr    ",
    "   rwwwwpwwwddwwwpwwwwr    ",
    "  rwwwwpppwwwwwwpppwwwwr   ",
    " rwwwwpppppwwwwpppppwwwwr  ",
];
const BUTLER: &[&str] = &[
    "         KKKKKKKKKK         ",
    "       KKKKKKKKKKKKKK       ",
    "      KKKKKKKKKKKKKKKK      ",
    "     KKKwwKKKKKKKKKKKKK     ",
    "     KKwwwwKKKKKKKKKKKK     ",
    "     KwwwwwwKKKKKKKKKKK     ",
    "     KwwsssssssssssssKK     ",
    "     KwwsssssssssssssKK     ",
    "     KwssKKssssssKKssKK     ",
    "     KwsswTsssssswTssKK     ",
    "      ssssTssssssTssss      ",
    "      sssssssshsssssss      ",
    "       ssssssshsssss       ",
    "       sssssmmmmssss       ",
    "        ssssssssss        ",
    "         ssssssss         ",
    "      dddwwsssswwddd      ",
    "    dddddwwwKKwwwddddd    ",
    "   dddddddwwKKwwddddddd   ",
    "  dddddddddwggwddddddddd  ",
    "  dddddddddwwwwddddddddd  ",
    " ddddddddddwwwwdddddddddd ",
    " ddddddddddwggwdddddddddd ",
    " ddddddddddwwwwdddddddddd ",
    " ddddddddddwwwwdddddddddd ",
    " ddddddddddwggwdddddddddd ",
];
const CAT_SMALL: &[&str] = &[
    " p      p ",
    "pppppppppp",
    "ppppwppppp",
    "ppssppsspp",
    "psTssssTsp",
    "pssbssbssp",
    " pssmmssp ",
    "  ddggdd  ",
];
const BUTLER_SMALL: &[&str] = &[
    " KKKKKKKK ",
    "KKwwKKKKKK",
    "KwwssssssK",
    "KssTssTssK",
    " ssssssss ",
    "  ssmmss  ",
    " ddwKKwdd ",
    "dddwwwwddd",
];
const CAT_MINI: &[&str] = &["p.pp.p", "psppsp", "pTssTp", " smms "];
const BUTLER_MINI: &[&str] = &["KKwwKK", "KsTTsK", " smms ", "dwKKwd"];
// A quiet lantern for old/custom profiles that haven't selected a portrait.
const NEUTRAL: &[&str] = &[
    "     gggggg     ",
    "    gwwwwwwg    ",
    "   gwwwwwwwwg   ",
    "   gwwggggwwg   ",
    "   gwwggggwwg   ",
    "   gwwggggwwg   ",
    "   gwwwwwwwwg   ",
    "    gggggggg    ",
    "      dddd      ",
];
pub fn portrait(frame: &mut Frame, area: Rect, theme: &Theme, blink: bool) {
    let art = match theme.portrait.as_str() {
        "cat" if area.width < 20 || area.height < 8 => CAT_MINI,
        "cat" if area.height <= 14 => CAT_SMALL,
        "cat" => CAT,
        "butler" if area.width < 20 || area.height < 8 => BUTLER_MINI,
        "butler" if area.height <= 14 => BUTLER_SMALL,
        "butler" => BUTLER,
        _ => NEUTRAL,
    };
    if area.width < 2 || area.height == 0 {
        return;
    }
    let source_width = art.iter().map(|r| r.len()).max().unwrap_or(1);
    // Contain-fit in square source pixels (two columns per terminal row).
    let scale = (f64::from(area.width / 2) / source_width as f64)
        .min(f64::from(area.height) / art.len() as f64)
        .min(1.0);
    let width = ((source_width as f64 * scale) as usize).max(1);
    let height = ((art.len() as f64 * scale) as usize).max(1);
    let x0 = area.x + (area.width.saturating_sub((width * 2) as u16)) / 2;
    let y0 = area.y + area.height.saturating_sub(height as u16);
    for y in 0..height {
        let sy = y * art.len() / height;
        for x in 0..width {
            let pixel = art[sy]
                .as_bytes()
                .get(x * source_width / width)
                .copied()
                .unwrap_or(b' ');
            let color = match pixel {
                b'r' => Color::Rgb(97, 43, 76),
                b'p' => theme.accent,
                b'h' => Color::Rgb(204, 105, 139),
                b'w' => theme.text,
                b's' => Color::Rgb(247, 200, 174),
                b'K' => Color::Rgb(32, 32, 47),
                b'T' if blink => Color::Rgb(32, 32, 47),
                b'T' => Color::Rgb(94, 143, 145),
                b'b' => Color::Rgb(234, 134, 151),
                b'm' => Color::Rgb(166, 65, 94),
                b'g' => theme.gold,
                b'd' => theme.surface,
                _ => continue,
            };
            for dx in 0..2 {
                if let Some(cell) = frame
                    .buffer_mut()
                    .cell_mut((x0 + x as u16 * 2 + dx, y0 + y as u16))
                {
                    cell.set_symbol(" ").set_style(Style::default().bg(color));
                }
            }
        }
    }
}
