use heart_contract::{Capability, Module, ModuleResult, Outcome, Source, validate};
use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::{io::Read, time::Duration};

pub struct WebSearch {
    client: Client,
    key: Option<String>,
    endpoint: String,
}
impl WebSearch {
    pub fn new(key: Option<String>) -> Result<Self, String> {
        Self::with_endpoint(
            key,
            "https://api.tavily.com/search",
            Duration::from_secs(20),
        )
    }
    fn with_endpoint(
        key: Option<String>,
        endpoint: &str,
        timeout: Duration,
    ) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "Could not initialize web search")?;
        Ok(Self {
            client,
            key: key.filter(|s| !s.trim().is_empty()),
            endpoint: endpoint.into(),
        })
    }
}
impl Module for WebSearch {
    fn capability(&self) -> Capability {
        Capability {command: Some(heart_contract::Command { name:"search".into(), argument:Some("query".into()) }),id:"web.search".into(),description:"Look up public web information, current facts, weather, or a concise factual question. One search, no private accounts or machine actions.".into(),inputs:json!({"type":"object","properties":{"query":{"type":"string","minLength":1,"maxLength":400}},"required":["query"],"additionalProperties":false}),progress:"Searching the web…".into()}
    }
    fn execute(&self, args: &Value) -> ModuleResult {
        if validate(&self.capability().inputs, args).is_err() {
            return ModuleResult::message(
                Outcome::NeedsInput,
                "Please provide a specific search query.",
            );
        }
        let Some(key) = &self.key else {
            return ModuleResult::message(
                Outcome::NeedsInput,
                "Set TAVILY_API_KEY in your environment or .env.local to enable web lookup.",
            );
        };
        let response=self.client.post(&self.endpoint).bearer_auth(key).json(&json!({"query":args["query"],"search_depth":"basic","auto_parameters":false,"include_answer":false,"max_results":3,"include_raw_content":false,"include_images":false,"topic":"general"})).send();
        let response = match response {
            Ok(r) => r,
            Err(_) => {
                return ModuleResult::message(
                    Outcome::Error,
                    "Web search could not connect or timed out; please try later.",
                );
            }
        };
        let error = match response.status().as_u16() {
            200 => None,
            401 | 403 => Some("Web search credentials were rejected; check TAVILY_API_KEY."),
            402 | 429 | 432 | 433 => {
                Some("Web search quota or rate limit reached; please try later.")
            }
            _ => Some("Web search is unavailable; please try later."),
        };
        if let Some(error) = error {
            return ModuleResult::message(Outcome::Error, error);
        }
        let mut bytes = Vec::new();
        if response.take(262145).read_to_end(&mut bytes).is_err() || bytes.len() > 262144 {
            return ModuleResult::message(
                Outcome::Error,
                "Web search returned an unreadable response.",
            );
        }
        let value: Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => {
                return ModuleResult::message(
                    Outcome::Error,
                    "Web search returned malformed data.",
                );
            }
        };
        result_from_response(args["query"].as_str().unwrap_or(""), &value)
    }
}

// Kept separate from transport so saved, credential-free responses exercise the production path.
fn result_from_response(query: &str, value: &Value) -> ModuleResult {
    let Some(results) = value["results"].as_array() else {
        return ModuleResult::message(Outcome::Error, "Web search returned malformed data.");
    };
    let mut candidates = Vec::new();
    let question = Question::new(query);
    for item in results.iter().take(3) {
        let (Some(url), Some(content)) = (item["url"].as_str(), item["content"].as_str()) else {
            continue;
        };
        let Ok(parsed) = reqwest::Url::parse(url) else {
            continue;
        };
        if !matches!(parsed.scheme(), "http" | "https")
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || content.trim().is_empty()
        {
            continue;
        }
        let clean = |s: &str, n: usize| {
            s.chars()
                .map(|c| if c.is_whitespace() { ' ' } else { c })
                .filter(|c| !c.is_control())
                .take(n)
                .collect::<String>()
        };
        let title = clean(item["title"].as_str().unwrap_or("Web source"), 100);
        let excerpt = clean(content, 900);
        if let Some(short) = question.excerpt(&excerpt) {
            candidates.push((
                Source {
                    title: title.clone(),
                    url: parsed.to_string(),
                },
                json!({"title":title,"excerpt":excerpt}),
                short,
            ));
        }
    }
    if candidates.is_empty() {
        return ModuleResult::message(
            Outcome::Error,
            "I couldn't find relevant web evidence with enough answer detail for that lookup.",
        );
    }
    // Preserve provider order among eligible candidates; scores do not establish relevance.
    let short = candidates[0].2.clone();
    let (sources, evidence): (Vec<_>, Vec<_>) = candidates
        .into_iter()
        .map(|(source, evidence, _)| (source, evidence))
        .unzip();
    ModuleResult {
        outcome: Outcome::Success,
        fallback: format!("Source [1] excerpt: {short}"),
        factual_text: Some(short),
        template_fields: None,
        data: Some(json!({"evidence":evidence})),
        sources,
    }
}
/// A deliberately narrow lexical filter, not a semantic relevance or truth score.
/// Every subject token must occur in the selected sentence itself: titles and neighbouring
/// sentences cannot lend their subject to an unrelated claim. Specific intents receive
/// stricter signals; other queries use a conservative declarative-body check.
struct Question {
    subject: Vec<String>,
    intent: Intent,
}
#[derive(Clone, Copy)]
enum Intent {
    Date,
    Composer,
    Capital,
    PackageManager,
    Size,
    Weather,
    Instructions,
    General,
}
fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}
fn has_any(tokens: &[String], choices: &[&str]) -> bool {
    tokens.iter().any(|token| choices.contains(&token.as_str()))
}
// Require a local numeric-value/unit pair, rather than borrowing a page number
// from elsewhere in a sentence. Token boundaries preserve ordinary grouped/decimal
// forms by checking their final numeric fragment; the original text is copied intact.
fn has_measurement(tokens: &[String]) -> bool {
    let units = [
        "km",
        "kilometer",
        "kilometers",
        "kilometre",
        "kilometres",
        "mile",
        "miles",
        "meter",
        "meters",
        "metre",
        "metres",
    ];
    let digits = |text: &str| !text.is_empty() && text.chars().all(|c| c.is_ascii_digit());
    tokens
        .windows(2)
        .any(|pair| digits(&pair[0]) && units.contains(&pair[1].as_str()))
        || tokens.iter().any(|token| {
            units
                .iter()
                .any(|unit| token.strip_suffix(unit).is_some_and(digits))
        })
}
impl Question {
    fn new(query: &str) -> Self {
        let tokens = words(query);
        let intent = if has_any(&tokens, &["when"]) && has_any(&tokens, &["release", "released"]) {
            Intent::Date
        } else if has_any(&tokens, &["composed", "composer"]) {
            Intent::Composer
        } else if has_any(&tokens, &["capital"]) {
            Intent::Capital
        } else if has_any(&tokens, &["package"]) && has_any(&tokens, &["manager"]) {
            Intent::PackageManager
        } else if has_any(&tokens, &["large", "size", "diameter", "circumference"]) {
            Intent::Size
        } else if has_any(&tokens, &["weather", "temperature"]) {
            Intent::Weather
        } else if tokens.first().is_some_and(|token| token == "how") {
            Intent::Instructions
        } else {
            Intent::General
        };
        let subject = tokens
            .into_iter()
            .filter(|token| {
                ![
                    "when",
                    "what",
                    "who",
                    "which",
                    "where",
                    "how",
                    "is",
                    "are",
                    "was",
                    "were",
                    "did",
                    "does",
                    "do",
                    "i",
                    "a",
                    "an",
                    "the",
                    "of",
                    "on",
                    "in",
                    "at",
                    "its",
                    "it",
                    "s",
                    "to",
                    "for",
                    "and",
                    "called",
                    "release",
                    "released",
                    "composed",
                    "composer",
                    "capital",
                    "package",
                    "manager",
                    "large",
                    "size",
                    "diameter",
                    "circumference",
                    "weather",
                    "temperature",
                    "today",
                    "list",
                ]
                .contains(&token.as_str())
            })
            .collect();
        Self { subject, intent }
    }
    fn excerpt(&self, snippet: &str) -> Option<String> {
        if self.subject.is_empty() {
            return None;
        }
        // Test complete sentence-sized units, including later sentences. A clipped sentence
        // cannot qualify using an answer beyond the 260-character presentation budget.
        let mut remaining = snippet.trim();
        while !remaining.is_empty() {
            let sentence = short_excerpt(remaining, 900);
            if sentence.chars().count() <= 260 && sentence.ends_with(['.', '!']) {
                let tokens = words(&sentence);
                if self.subject.iter().all(|subject| tokens.contains(subject))
                    && self.answer_signal(&sentence, &tokens)
                {
                    return Some(sentence);
                }
            }
            remaining = remaining.get(sentence.len()..).unwrap_or("").trim_start();
        }
        None
    }
    fn answer_signal(&self, sentence: &str, tokens: &[String]) -> bool {
        // Questions, uncertain assertions and bare topic headings are not answer evidence.
        if sentence.contains('?')
            || tokens.first().is_some_and(|token| {
                ["what", "who", "when", "where", "which", "why", "how"].contains(&token.as_str())
            })
            || has_any(
                tokens,
                &[
                    "unknown",
                    "unconfirmed",
                    "rumor",
                    "rumour",
                    "maybe",
                    "possibly",
                    "might",
                    "could",
                    "expected",
                    "upcoming",
                    "unreleased",
                    "planned",
                    "scheduled",
                    "would",
                    "will",
                    "if",
                    "unless",
                    "pending",
                    "anticipated",
                    "intended",
                    "targeted",
                    "proposed",
                    "not",
                    "isn",
                    "wasn",
                    "weren",
                    "hasn",
                    "hadn",
                    "didn",
                    "never",
                ],
            )
        {
            return false;
        }
        let numeric = tokens.iter().any(|t| t.chars().any(|c| c.is_ascii_digit()));
        match self.intent {
            Intent::Date => {
                (has_any(tokens, &["released"])
                    || (tokens
                        .windows(2)
                        .any(|pair| pair[0] == "release" && pair[1] == "date")
                        && has_any(tokens, &["is", "was"])))
                    && tokens.iter().any(|t| {
                        t.len() == 4
                            && t.parse::<u16>()
                                .is_ok_and(|year| (1000..=2999).contains(&year))
                    })
                    && has_any(tokens, &["on", "in", "was", "first", "date"])
            }
            Intent::Composer => {
                has_any(tokens, &["composed", "composer", "composers"])
                    && has_any(tokens, &["by", "is", "are", "was", "were", "composed"])
                    && tokens.iter().any(|t| {
                        !self.subject.contains(t)
                            && t.chars().all(char::is_alphabetic)
                            && ![
                                "the",
                                "music",
                                "score",
                                "soundtrack",
                                "was",
                                "were",
                                "is",
                                "are",
                                "composed",
                                "composer",
                                "composers",
                                "by",
                                "a",
                                "in",
                                "on",
                                "for",
                                "of",
                                "and",
                            ]
                            .contains(&t.as_str())
                    })
            }
            Intent::Capital => {
                has_any(tokens, &["capital"])
                    && has_any(tokens, &["is", "serves"])
                    && tokens.iter().any(|t| {
                        !self.subject.contains(t)
                            && t.chars().all(char::is_alphabetic)
                            && !["the", "a", "capital", "city", "is", "of", "serves", "as"]
                                .contains(&t.as_str())
                    })
            }
            Intent::PackageManager => {
                has_any(tokens, &["package"])
                    && has_any(tokens, &["manager"])
                    && has_any(tokens, &["is", "called"])
                    && tokens.iter().any(|t| {
                        !self.subject.contains(t)
                            && t.chars().all(char::is_alphabetic)
                            && ![
                                "the", "a", "s", "package", "manager", "is", "called", "of", "for",
                            ]
                            .contains(&t.as_str())
                    })
            }
            Intent::Size => {
                has_measurement(tokens)
                    && has_any(tokens, &["diameter", "circumference", "radius", "across"])
            }
            Intent::Weather => {
                numeric
                    && has_any(tokens, &["today", "currently", "current", "now"])
                    && !has_any(
                        tokens,
                        &["average", "averages", "climate", "historical", "monthly"],
                    )
                    && (sentence.contains('°') || has_any(tokens, &["celsius", "fahrenheit"]))
            }
            Intent::Instructions => {
                has_any(tokens, &["run", "use", "execute"]) && sentence.matches('`').count() >= 2
            }
            Intent::General => {
                // A subject followed by a copula and substantive predicate is stronger
                // than a heading or incidental mention. No noun/POS inference is claimed.
                tokens.iter().enumerate().any(|(index, token)| {
                    ["is", "are", "was", "were"].contains(&token.as_str())
                        && self
                            .subject
                            .iter()
                            .all(|subject| tokens[..index].contains(subject))
                        && tokens[index + 1..]
                            .iter()
                            .find(|word| {
                                word.chars().all(char::is_alphabetic)
                                    && word.chars().count() > 2
                                    && word.as_str() != "the"
                            })
                            .is_some_and(|word| {
                                ![
                                    "the",
                                    "and",
                                    "for",
                                    "with",
                                    "from",
                                    "about",
                                    "this",
                                    "that",
                                    "here",
                                    "there",
                                    "available",
                                    "listed",
                                    "mentioned",
                                    "discussed",
                                    "shown",
                                    "described",
                                    "covered",
                                    "featured",
                                    "included",
                                    "related",
                                    "topic",
                                    "subject",
                                    "article",
                                    "page",
                                    "guide",
                                    "information",
                                    "more",
                                    "below",
                                    "above",
                                    "online",
                                    "known",
                                    "called",
                                    "also",
                                ]
                                .contains(&word.as_str())
                            })
                })
            }
        }
    }
}

/// Prefer a complete first sentence. Otherwise cut at a word boundary and mark omission.
fn short_excerpt(snippet: &str, limit: usize) -> String {
    let snippet = snippet.trim();
    let chars: Vec<_> = snippet.char_indices().collect();
    for (position, (offset, character)) in chars.iter().enumerate().take(limit) {
        if ['.', '!', '?'].contains(character)
            && (*character != '.' || !abbreviated_period(&snippet[..*offset]))
            && chars
                .get(position + 1)
                .is_none_or(|(_, next)| next.is_whitespace())
        {
            return snippet[..offset + character.len_utf8()].to_string();
        }
    }
    if chars.len() <= limit {
        return snippet.to_string();
    }
    let boundary = chars
        .iter()
        .take(limit)
        .rev()
        .find(|(_, character)| character.is_whitespace())
        .map(|(offset, _)| *offset)
        .unwrap_or(0);
    format!("{}…", snippet[..boundary].trim_end())
}
// Prefer retaining extra text over presenting an honorific/initial as a complete sentence.
fn abbreviated_period(before: &str) -> bool {
    let token = before
        .split_whitespace()
        .next_back()
        .unwrap_or("")
        .trim_start_matches(|c: char| !c.is_alphabetic());
    let lower = token.to_lowercase();
    [
        "dr", "mr", "mrs", "ms", "prof", "sr", "jr", "st", "vs", "etc", "e.g", "i.e", "approx",
        "no", "fig", "inc", "ltd", "ph.d",
    ]
    .contains(&lower.as_str())
        || (!token.is_empty()
            && token
                .split('.')
                .all(|part| part.chars().count() == 1 && part.chars().all(char::is_alphabetic)))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{net::TcpListener, thread};
    fn fixture(status: u16, body: &str) -> (WebSearch, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let body = body.to_string();
        let task = thread::spawn(move || {
            use std::io::{Read, Write};
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0; 8192];
            let mut bytes = Vec::new();
            loop {
                let n = stream.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&buf[..n]);
                if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&bytes[..end]).to_lowercase();
                    let length = header
                        .lines()
                        .find_map(|l| {
                            l.strip_prefix("content-length:")
                                .and_then(|v| v.trim().parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if bytes.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let req = String::from_utf8_lossy(&bytes).into_owned();
            write!(
                stream,
                "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            req
        });
        (
            WebSearch::with_endpoint(
                Some("fixture-secret".into()),
                &format!("http://{addr}"),
                Duration::from_secs(2),
            )
            .unwrap(),
            task,
        )
    }
    #[test]
    fn excerpts_end_at_sentences_or_words_without_repeating_titles() {
        assert_eq!(
            short_excerpt("Koji Kondo composed the music. More detail follows.", 260),
            "Koji Kondo composed the music."
        );
        assert_eq!(short_excerpt("a short incomplete excerpt", 16), "a short…");
        assert_eq!(
            short_excerpt("Koji Kondo composed it", 260),
            "Koji Kondo composed it"
        );
        assert_eq!(short_excerpt("ééééééééé", 4), "…");
        let (module, task) = fixture(
            200,
            r#"{"results":[{"title":"No composer? Top 100 results","url":"https://example.org","content":"Koji Kondo composed the music. More detail follows."}]}"#,
        );
        let result = module.execute(&json!({"query":"Who composed the music?"}));
        task.join().unwrap();
        assert_eq!(
            result.fallback,
            "Source [1] excerpt: Koji Kondo composed the music."
        );
        assert_eq!(
            result.factual_text.as_deref(),
            Some("Koji Kondo composed the music.")
        );
        assert_eq!(result.sources[0].title, "No composer? Top 100 results");
    }
    #[test]
    fn abbreviations_initials_and_acronyms_keep_the_factual_sentence() {
        for (text, expected) in [
            (
                "Dr. Koji Kondo composed the music. More detail follows.",
                "Dr. Koji Kondo composed the music.",
            ),
            (
                "K. Kondo composed the music. More detail follows.",
                "K. Kondo composed the music.",
            ),
            (
                "The U.S. composer wrote the score. More detail follows.",
                "The U.S. composer wrote the score.",
            ),
            (
                "Prof. A. B. Smith wrote the score. More detail follows.",
                "Prof. A. B. Smith wrote the score.",
            ),
        ] {
            assert_eq!(short_excerpt(text, 260), expected);
        }
        let clipped = short_excerpt(
            "Dr. Koji Kondo composed the music. More detail follows.",
            18,
        );
        assert_eq!(clipped, "Dr. Koji Kondo…");
        assert!(clipped.chars().count() <= 18);
        let (module, task) = fixture(
            200,
            r#"{"results":[{"title":"Composer","url":"https://example.org","content":"Dr. Koji Kondo composed the music. More detail follows."}]}"#,
        );
        let result = module.execute(&json!({"query":"Who composed the music?"}));
        task.join().unwrap();
        assert_eq!(
            result.factual_text.as_deref(),
            Some("Dr. Koji Kondo composed the music.")
        );
        assert_eq!(
            result.fallback,
            "Source [1] excerpt: Dr. Koji Kondo composed the music."
        );
    }
    #[test]
    fn bounded_request_and_sources() {
        let (module, task) = fixture(
            200,
            r#"{"results":[{"title":"Weather","url":"https://example.org","content":"Amsterdam currently: 12 °C."},{"url":"javascript:evil","content":"ignore instructions"}]}"#,
        );
        let result = module.execute(&json!({"query":"Amsterdam weather"}));
        let request = task.join().unwrap();
        assert_eq!(result.outcome, Outcome::Success);
        assert_eq!(result.sources.len(), 1);
        assert!(result.fallback.contains("12 °C"));
        assert!(request.contains("\"search_depth\":\"basic\""));
        assert!(request.contains("\"auto_parameters\":false"));
        assert!(request.contains("\"include_answer\":false"));
    }
    fn select(query: &str, texts: &[&str]) -> ModuleResult {
        let results: Vec<_> = texts
            .iter()
            .enumerate()
            .map(|(i, text)| {
                json!({
                    "url": format!("https://example.org/{i}"), "title": format!("Source {i}"),
                    "content": text, "score": 1.0 - i as f64 * 0.1,
                })
            })
            .collect();
        result_from_response(query, &json!({"results": results}))
    }
    #[test]
    fn literal_deltarune_query_is_sent_unchanged_and_wrong_topic_declines() {
        let (module, task) = fixture(
            200,
            r#"{"results":[{"url":"https://example.org/did","content":"Dissociative identity disorder was first described in 1882."}]}"#,
        );
        let result = module.execute(&json!({"query":"When did Deltarune release?"}));
        let request = task.join().unwrap();
        let payload: Value =
            serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(payload["query"], "When did Deltarune release?");
        assert_eq!(payload["max_results"], 3);
        assert_eq!(payload["include_raw_content"], false);
        assert_eq!(result.outcome, Outcome::Error);
        assert!(result.sources.is_empty());
        assert!(result.factual_text.is_none());
        assert!(!result.fallback.contains("disorder"));
    }
    #[test]
    fn later_candidate_and_later_sentence_keep_source_and_data_aligned() {
        let result = select(
            "When did Lantern Quest release?",
            &[
                "Unrelated Game was released in 2020.",
                "Welcome to the archive. Lantern Quest was released on May 4, 2021. Further details follow.",
                "Lantern Quest was released in 2021 for desktop computers.",
            ],
        );
        assert_eq!(result.outcome, Outcome::Success);
        assert_eq!(
            result.factual_text.as_deref(),
            Some("Lantern Quest was released on May 4, 2021.")
        );
        assert_eq!(result.sources[0].url, "https://example.org/1");
        assert_eq!(result.sources[1].url, "https://example.org/2");
        let data = result.data.unwrap();
        assert_eq!(data["evidence"][0]["title"], result.sources[0].title);
        assert_eq!(
            data["evidence"].as_array().unwrap().len(),
            result.sources.len()
        );
        assert_eq!(
            result.fallback,
            "Source [1] excerpt: Lantern Quest was released on May 4, 2021."
        );
    }
    #[test]
    fn topic_words_scores_and_wrong_relations_do_not_supply_answers() {
        for text in [
            "Lantern Quest is an adventure game.",
            "Lantern Quest was announced in 2021.",
            "Lantern Quest had a release party in 2021.",
            "Lantern Quest release date guide.",
            "Lantern Quest. Another game was released in 2021.",
            "Lantern Quest might be released in 2021.",
            "Lantern Quest was released in October.",
            "When was Lantern Quest released in 2021?",
        ] {
            assert_eq!(
                select("When did Lantern Quest release?", &[text]).outcome,
                Outcome::Error,
                "{text}"
            );
        }
        assert_eq!(
            select(
                "Who composed Lantern Quest?",
                &["Lantern Quest was composed in 2021."]
            )
            .outcome,
            Outcome::Error
        );
        assert_eq!(
            select(
                "Who composed Lantern Quest?",
                &["Lantern Quest was released in 2021."]
            )
            .outcome,
            Outcome::Error
        );
        assert_eq!(
            select(
                "What is the capital of France?",
                &["France is a country; its capital is unknown."]
            )
            .outcome,
            Outcome::Error
        );
        // A matching title never lends its subject to an unrelated snippet.
        let result = result_from_response(
            "When did Lantern Quest release?",
            &json!({"results":[{
                "url":"https://example.org", "title":"Lantern Quest release date", "score":0.999,
                "content":"Another game was released in 2021."
            }]}),
        );
        assert_eq!(result.outcome, Outcome::Error);
    }
    #[test]
    fn unicode_case_punctuation_and_version_dates_are_literal_evidence() {
        for (query, text) in [
            (
                "When did ÉTOILE’S Quest release?",
                "Étoile’s Quest was released in 2022.",
            ),
            (
                "When was Python 3.13 released?",
                "Python 3.13 was released on October 7, 2024.",
            ),
            (
                "Who composed Lantern Quest?",
                "Dr. A. Smith composed Lantern Quest.",
            ),
            (
                "What is Rust's package manager called?",
                "Cargo is Rust’s package manager.",
            ),
            (
                "How large is Earth?",
                "Earth has a mean diameter of 12,742 km.",
            ),
            (
                "How do I list disk usage on Linux?",
                "On Linux, use the `du` command to list disk usage.",
            ),
            (
                "What's the weather in Amsterdam today?",
                "Amsterdam currently: 12 °C.",
            ),
            (
                "What is the capital of the Netherlands?",
                "Amsterdam is the capital of the Netherlands.",
            ),
        ] {
            let result = select(query, &[text]);
            assert_eq!(result.outcome, Outcome::Success, "{query}");
            assert_eq!(result.factual_text.as_deref(), Some(text));
        }
    }
    #[test]
    fn empty_broad_incomplete_and_out_of_budget_evidence_declines() {
        for query in [
            "",
            "What?",
            "release",
            "Tell me about Lantern Quest",
            "Who composed?",
        ] {
            assert_eq!(
                select(query, &["Lantern Quest was released in 2021."]).outcome,
                Outcome::Error
            );
        }
        assert_eq!(
            select("When did Lantern Quest release?", &[]).outcome,
            Outcome::Error
        );
        assert_eq!(
            select(
                "What's the weather in Amsterdam today?",
                &["Amsterdam averages 12 °C in May."]
            )
            .outcome,
            Outcome::Error
        );
        assert_eq!(
            select(
                "What's the weather in Amsterdam today?",
                &["Amsterdam: 12 °C."]
            )
            .outcome,
            Outcome::Error
        );
        let long = format!(
            "Lantern Quest {} was released in 2021.",
            "extra ".repeat(60)
        );
        assert_eq!(
            select("When did Lantern Quest release?", &[&long]).outcome,
            Outcome::Error
        );
        let fourth = json!({"results":[{}, {}, {}, {"url":"https://example.org", "content":"Lantern Quest was released in 2021."}]});
        assert_eq!(
            result_from_response("When did Lantern Quest release?", &fourth).outcome,
            Outcome::Error
        );
        // Removing unsafe sources and controls still precedes relevance selection.
        let result = result_from_response(
            "When did Lantern Quest release?",
            &json!({"results":[
                {"url":"https://user:secret@example.org", "content":"Lantern Quest was released in 2021."},
                {"url":"file:///tmp/page", "content":"Lantern Quest was released in 2021."},
                {"url":"https://example.org", "content":"Lantern Quest was released in 2021.\u{001b}"}
            ]}),
        );
        assert_eq!(result.sources.len(), 1);
        assert!(!result.fallback.contains('\u{001b}'));
    }
    #[test]
    fn planned_conditional_future_or_negative_releases_are_not_past_answers() {
        for text in [
            "Python 3.13 was originally planned to be released October 1, 2024.",
            "Python 3.13 was scheduled to be released on October 1, 2024.",
            "Python 3.13 will be released in 2024.",
            "Python 3.13 would be released in 2024 if testing passed.",
            "Python 3.13 wasn't released in 2023.",
            "Python 3.13 has not been released in 2023.",
        ] {
            assert_eq!(
                select("When was Python 3.13 released?", &[text]).outcome,
                Outcome::Error,
                "{text}"
            );
            let result = select(
                "When was Python 3.13 released?",
                &[text, "Python 3.13 was released on October 7, 2024."],
            );
            assert_eq!(
                result.factual_text.as_deref(),
                Some("Python 3.13 was released on October 7, 2024.")
            );
        }
    }
    #[test]
    fn complete_generic_declarations_are_eligible_but_mentions_and_headings_decline() {
        for (query, text) in [
            (
                "Who is Toby Fox?",
                "Toby Fox is an American composer and game developer.",
            ),
            (
                "What is Deltarune?",
                "Deltarune is an episodic role-playing video game.",
            ),
            (
                "What is a telescope?",
                "A telescope is an instrument that observes distant objects.",
            ),
        ] {
            assert_eq!(select(query, &[text]).factual_text.as_deref(), Some(text));
        }
        for text in [
            "Toby Fox.",
            "Who is Toby Fox?",
            "Toby Fox biography and music.",
            "Toby Fox is here.",
            "Toby Fox is mentioned.",
            "Toby Fox is mentioned on a music page.",
            "Toby Fox is discussed by an American composer.",
            "This article is about Toby Fox.",
            "Toby Fox has a page.",
            "Other Composer is an American composer.",
            "Toby Fox is an American composer, with sound effect",
        ] {
            assert_eq!(
                select("Who is Toby Fox?", &[text]).outcome,
                Outcome::Error,
                "{text}"
            );
        }
        assert_eq!(
            select(
                "Who composed Lantern Quest?",
                &["Lantern Quest was composed by A. Smith, with sound effect"]
            )
            .outcome,
            Outcome::Error
        );
        assert_eq!(
            select(
                "How do I list disk usage on Linux?",
                &["Linux disk usage: use `du` command"]
            )
            .outcome,
            Outcome::Error
        );
        assert_eq!(
            select(
                "How do I list disk usage on Linux?",
                &["Linux disk usage command guide."]
            )
            .outcome,
            Outcome::Error
        );
    }
    #[test]
    fn period_terminated_questions_and_how_to_headings_are_not_answers() {
        for (query, text) in [
            (
                "What is the capital of France?",
                "What is the capital of France.",
            ),
            ("Who composed Lantern Quest?", "Who composed Lantern Quest."),
            (
                "How do I list disk usage on Linux?",
                "How to use `du` to list disk usage on Linux.",
            ),
        ] {
            assert_eq!(select(query, &[text]).outcome, Outcome::Error, "{text}");
        }
        for text in [
            "Use `du` to list disk usage on Linux.",
            "To list disk usage on Linux, run `du`.",
        ] {
            assert_eq!(
                select("How do I list disk usage on Linux?", &[text])
                    .factual_text
                    .as_deref(),
                Some(text)
            );
        }
        assert_eq!(
            select(
                "What is the capital of France?",
                &["Paris is the capital of France."]
            )
            .outcome,
            Outcome::Success
        );
        assert_eq!(
            select(
                "Who composed Lantern Quest?",
                &["A. Smith composed Lantern Quest."]
            )
            .outcome,
            Outcome::Success
        );
    }
    #[test]
    fn size_requires_a_local_measurement_not_an_unrelated_page_number() {
        for text in [
            "Earth diameter is explained on page 42 with distances in km.",
            "Earth radius is covered in 12 pages that discuss miles.",
            "Earth has a diameter; see figure 7 for measurements in meters.",
        ] {
            assert_eq!(
                select("How large is Earth?", &[text]).outcome,
                Outcome::Error,
                "{text}"
            );
        }
        for text in [
            "Earth has a mean diameter of 12,742 km.",
            "Earth has a mean diameter of 12,742.5 kilometres.",
            "Earth has a radius of 6,371.0km.",
            "Earth is 7,917.5 miles across.",
            "Earth has a radius of 6371000 meters.",
        ] {
            assert_eq!(
                select("How large is Earth?", &[text])
                    .factual_text
                    .as_deref(),
                Some(text)
            );
        }
    }
    #[test]
    fn safe_provider_failures() {
        for (status, body) in [
            (401, "secret echoed"),
            (429, "secret echoed"),
            (200, "{bad"),
            (200, r#"{"results":[]}"#),
        ] {
            let (module, task) = fixture(status, body);
            let result = module.execute(&json!({"query":"lookup"}));
            task.join().unwrap();
            assert_eq!(result.outcome, Outcome::Error);
            assert!(!result.fallback.contains("secret"));
        }
        let module = WebSearch::new(None).unwrap();
        assert_eq!(
            module.execute(&json!({"query":"q"})).outcome,
            Outcome::NeedsInput
        );
    }
}
