//! Command classification shared by both frontends; execution remains in the core.
use heart_core::{Engine, Inference, Personality, Reply};
use std::collections::BTreeMap;

pub enum Action {
    Empty,
    Exit,
    Help,
    Profile(String),
    Request(String),
    Message(String),
}
pub struct Session {
    pub profiles: BTreeMap<String, Personality>,
    pub selected: String,
    pub help: String,
    pub experimental_nl: bool,
    pub diagnostics: bool,
}
impl Session {
    pub fn profile(&self) -> &Personality {
        &self.profiles[&self.selected]
    }
    pub fn classify(&mut self, line: &str) -> Action {
        let request = line.trim();
        match request {
            "" => return Action::Empty,
            "/exit" | "/quit" => return Action::Exit,
            "/help" => return Action::Help,
            _ => {}
        }
        let id = request
            .strip_prefix("/profile ")
            .map(str::trim)
            .or_else(|| {
                request
                    .strip_prefix('/')
                    .filter(|id| self.profiles.contains_key(*id))
            });
        if let Some(id) = id {
            if self.profiles.contains_key(id) {
                self.selected = id.to_owned();
                return Action::Profile(self.profile().greeting().to_owned());
            }
            return Action::Message("Unknown profile; see /help.".into());
        }
        if !request.starts_with('/') && !self.experimental_nl {
            return Action::Message("Use a slash command; see /help. Natural-language routing requires --experimental-nl.".into());
        }
        Action::Request(request.to_owned())
    }
    pub fn cycle(&mut self) {
        let keys: Vec<_> = self.profiles.keys().cloned().collect();
        let next = (keys.iter().position(|id| id == &self.selected).unwrap_or(0) + 1) % keys.len();
        self.selected = keys[next].clone();
    }
}
pub fn execute<I: Inference>(
    engine: &mut Engine<I>,
    request: &str,
    profile: &Personality,
    progress: impl Fn(&str),
) -> Result<Reply, String> {
    if request.starts_with('/') {
        engine.invoke(request, profile, progress)
    } else {
        engine.ask(request, profile, progress)
    }
}
pub fn diagnostic(reply: &Reply) -> String {
    format!(
        "diagnostics: phrase={:?} routing_performed={} selection_ms={} extraction_ms={} route_ms={} phrase_ms={} total_ms={}",
        reply.phrase_status,
        reply.routing_performed,
        reply.selection_timing.elapsed_ms,
        reply
            .extraction_timing
            .as_ref()
            .map(|t| t.elapsed_ms)
            .unwrap_or(0),
        reply.route_timing.elapsed_ms,
        reply
            .phrase_timing
            .as_ref()
            .map(|t| t.elapsed_ms)
            .unwrap_or(0),
        reply.elapsed_ms
    )
}
pub fn reply_lines(reply: Reply, diagnostics: bool) -> Vec<String> {
    let mut lines = vec![reply.text.clone()];
    for (i, source) in reply.sources.iter().enumerate() {
        lines.push(format!("[{}] {} — {}", i + 1, source.title, source.url));
    }
    if diagnostics {
        lines.push(diagnostic(&reply));
    }
    if let Some(warning) = reply.warning {
        lines.push(warning);
    }
    lines
}
