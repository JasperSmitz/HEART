use heart_core::Personality;
use ratatui::style::Color;

pub struct Theme {
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub accent: Color,
    pub gold: Color,
    pub portrait: String,
    pub name: String,
}
impl Theme {
    pub fn for_profile(profile: &Personality) -> Self {
        let Some(a) = &profile.appearance else {
            return Self {
                background: Color::Rgb(25, 28, 35),
                surface: Color::Rgb(36, 40, 49),
                text: Color::Rgb(243, 233, 215),
                accent: Color::Rgb(165, 188, 194),
                gold: Color::Rgb(215, 185, 137),
                portrait: "neutral".into(),
                name: profile.id.clone(),
            };
        };
        fn color(hex: &str) -> Color {
            let value = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0);
            Color::Rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
        }
        Self {
            background: color(&a.background),
            surface: color(&a.surface),
            text: color(&a.text),
            accent: color(&a.accent),
            gold: color(&a.gold),
            portrait: a.portrait.clone(),
            name: a.display_name.clone(),
        }
    }
}
