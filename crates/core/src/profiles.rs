use heart_contract::{BUILTIN_COMMANDS, valid_name};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// Trusted local presentation configuration, never capability or guard authority.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Personality {
    pub id: String,
    pub instructions: String,
    pub greeting: String,
    pub status: String,
    #[serde(default)]
    pub progress: BTreeMap<String, String>,
    #[serde(default)]
    pub templates: BTreeMap<String, String>,
    pub presentation: Option<Presentation>,
    pub appearance: Option<Appearance>,
}
/// Optional frontend-neutral styling. Hex colors and builtin asset keys are validated on load.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    pub portrait: String,
    pub display_name: String,
    pub background: String,
    pub surface: String,
    pub text: String,
    pub accent: String,
    pub gold: String,
}
impl Appearance {
    fn valid(&self) -> bool {
        matches!(self.portrait.as_str(), "cat" | "butler" | "neutral")
            && !self.display_name.trim().is_empty()
            && !self.display_name.chars().any(char::is_control)
            && [
                &self.background,
                &self.surface,
                &self.text,
                &self.accent,
                &self.gold,
            ]
            .iter()
            .all(|color| {
                color.len() == 7
                    && color.starts_with('#')
                    && color[1..].bytes().all(|b| b.is_ascii_hexdigit())
            })
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Presentation {
    pub suffix: String,
    pub markers: Vec<String>,
}
impl Personality {
    // Convenience constructors for fixtures/examples; the CLI discovers files at runtime.
    pub fn butler() -> Self {
        serde_json::from_str(include_str!("../../../profiles/butler.json"))
            .expect("bundled profile")
    }
    pub fn cat() -> Self {
        serde_json::from_str(include_str!("../../../profiles/cat.json")).expect("bundled profile")
    }
    pub fn name(&self) -> &str {
        &self.id
    }
    pub fn greeting(&self) -> &str {
        &self.greeting
    }
    pub fn routing_status(&self) -> &str {
        &self.status
    }
    pub fn instruction(&self) -> &str {
        &self.instructions
    }
    pub fn present(&self, answer: &str) -> String {
        let Some(presentation) = &self.presentation else {
            return answer.into();
        };
        let words: Vec<_> = answer
            .split(|c: char| !c.is_alphanumeric())
            .map(str::to_lowercase)
            .collect();
        if presentation
            .markers
            .iter()
            .any(|marker| words.contains(marker))
        {
            return answer.into();
        }
        format!(
            "{}, {}.",
            answer.trim_end_matches(['.', '!', '?']).trim_end(),
            presentation.suffix
        )
    }
}
pub fn load_profiles(
    directory: &Path,
    commands: impl IntoIterator<Item = String>,
) -> Result<BTreeMap<String, Personality>, String> {
    let reserved: BTreeSet<String> = BUILTIN_COMMANDS
        .iter()
        .map(|s| s.to_string())
        .chain(commands)
        .collect();
    let mut profiles = BTreeMap::new();
    let mut paths = std::fs::read_dir(directory)
        .map_err(|_| "Could not read profiles directory")?
        .map(|entry| {
            entry
                .map(|e| e.path())
                .map_err(|_| "Could not read profile entry")
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    for path in paths {
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|_| format!("Could not read profile {}", path.display()))?;
        let profile: Personality = serde_json::from_str(&text)
            .map_err(|_| format!("Invalid profile JSON: {}", path.display()))?;
        let safe = |s: &str| !s.trim().is_empty() && !s.chars().any(char::is_control);
        if !valid_name(&profile.id)
            || reserved.contains(&profile.id)
            || profiles.contains_key(&profile.id)
        {
            return Err(format!(
                "Duplicate, invalid or reserved profile ID in {}",
                path.display()
            ));
        }
        if profile.appearance.as_ref().is_some_and(|a| !a.valid()) {
            return Err(format!("Invalid profile appearance in {}", path.display()));
        }
        if !safe(&profile.instructions)
            || !safe(&profile.greeting)
            || !safe(&profile.status)
            || profile.progress.values().any(|s| !safe(s))
            || profile
                .presentation
                .as_ref()
                .is_some_and(|p| !safe(&p.suffix) || p.markers.iter().any(|m| !valid_name(m)))
        {
            return Err(format!("Invalid profile text in {}", path.display()));
        }
        profiles.insert(profile.id.clone(), profile);
    }
    if profiles.is_empty() {
        return Err("Profiles directory contains no JSON profiles".into());
    }
    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Directory(std::path::PathBuf);
    impl Directory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "heart-profiles-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn write(&self, name: &str, value: &str) {
            std::fs::write(self.0.join(name), value).unwrap();
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn pirate() -> String {
        serde_json::json!({"id":"pirate","instructions":"A concise pirate","greeting":"Ahoy!","status":"Checking the charts.","progress":{"opaque.capability":"Consulting the charts."}}).to_string()
    }
    #[test]
    fn discovers_custom_profile_and_preserves_authored_text() {
        let directory = Directory::new();
        directory.write("arbitrary-filename.json", &pirate());
        directory.write("ignored.md", "not parsed");
        let profiles = load_profiles(&directory.0, ["search".into()]).unwrap();
        let profile = &profiles["pirate"];
        assert!(profile.appearance.is_none());
        assert!(profile.templates.is_empty()); // Older JSON profiles remain valid.
        assert_eq!(profile.greeting(), "Ahoy!");
        assert_eq!(profile.instruction(), "A concise pirate");
        assert_eq!(profile.routing_status(), "Checking the charts.");
        assert_eq!(
            profile.progress["opaque.capability"],
            "Consulting the charts."
        );
        assert_eq!(
            profile.present("Amsterdam is 12 °C."),
            "Amsterdam is 12 °C."
        );
        let bundled = load_profiles(
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../profiles")),
            ["search".into()],
        )
        .unwrap();
        assert_eq!(
            bundled["butler"].greeting(),
            Personality::butler().greeting()
        );
        assert_eq!(bundled["cat"].instructions, Personality::cat().instructions);
    }
    #[test]
    fn custom_templates_load_without_rewriting_other_profile_text() {
        let directory = Directory::new();
        let mut profile: serde_json::Value = serde_json::from_str(&pirate()).unwrap();
        profile["templates"] = serde_json::json!({"opaque.capability":"Arrr, {quantity}!"});
        directory.write("pirate.json", &profile.to_string());
        let profiles = load_profiles(&directory.0, Vec::<String>::new()).unwrap();
        assert_eq!(
            profiles["pirate"].templates["opaque.capability"],
            "Arrr, {quantity}!"
        );
        assert_eq!(profiles["pirate"].instructions, "A concise pirate");
    }
    #[test]
    fn optional_appearance_is_plain_validated_profile_data() {
        let directory = Directory::new();
        let mut value: serde_json::Value = serde_json::from_str(&pirate()).unwrap();
        let appearance = serde_json::json!({"portrait":"cat","display_name":"Captain", "background":"#201923","surface":"#302330","text":"#ffe9d4","accent":"#ef9bae","gold":"#e7c584"});
        value["appearance"] = appearance.clone();
        directory.write("pirate.json", &value.to_string());
        let profiles = load_profiles(&directory.0, Vec::<String>::new()).unwrap();
        assert_eq!(
            profiles["pirate"].appearance.as_ref().unwrap().portrait,
            "cat"
        );
        for (field, invalid) in [
            ("portrait", "external.png"),
            ("accent", "red"),
            ("background", "#xyz123"),
            ("display_name", "bad\u{1b}name"),
        ] {
            value["appearance"] = appearance.clone();
            value["appearance"][field] = invalid.into();
            directory.write("pirate.json", &value.to_string());
            assert!(
                load_profiles(&directory.0, Vec::<String>::new())
                    .unwrap_err()
                    .contains("appearance")
            );
        }
    }
    #[test]
    fn malformed_duplicate_and_reserved_profiles_fail_clearly() {
        for (text, second, commands) in [
            ("{bad".to_string(), None, vec![]),
            (pirate(), Some(pirate()), vec![]),
            (pirate().replace("pirate", "help"), None, vec![]),
            (
                pirate().replace("pirate", "search"),
                None,
                vec!["search".into()],
            ),
            (pirate().replace("pirate", "Bad ID"), None, vec![]),
            (pirate().replace("Ahoy!", ""), None, vec![]),
            (pirate().replace("Ahoy!", "bad\\u001btext"), None, vec![]),
            (
                pirate().replace("\"id\":", "\"unknown\":1,\"id\":"),
                None,
                vec![],
            ),
        ] {
            let directory = Directory::new();
            directory.write("one.json", &text);
            if let Some(second) = second {
                directory.write("two.json", &second);
            }
            assert!(load_profiles(&directory.0, commands).is_err(), "{text}");
        }
        let directory = Directory::new();
        assert!(load_profiles(&directory.0, Vec::<String>::new()).is_err());
        assert!(load_profiles(&directory.0.join("missing"), Vec::<String>::new()).is_err());
    }
}
