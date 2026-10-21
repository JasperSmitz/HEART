use super::{Completion, Inference, InferenceError, Timing};
use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

pub struct Ollama {
    client: Client,
    endpoint: String,
    model: String,
    warm_timeout: Duration,
    cold_timeout: Duration,
}
impl Ollama {
    pub fn new(
        endpoint: &str,
        model: &str,
        warm_timeout: Duration,
        cold_timeout: Duration,
    ) -> Result<Self, String> {
        let url = reqwest::Url::parse(endpoint).map_err(|_| "Invalid Ollama endpoint")?;
        if !matches!(url.scheme(), "http" | "https")
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(
                "Ollama endpoint must be an HTTP(S) URL without credentials, query or fragment"
                    .into(),
            );
        }
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(3))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "Cannot create local inference client")?;
        Ok(Self {
            client,
            endpoint: endpoint.trim_end_matches('/').into(),
            model: model.into(),
            warm_timeout,
            cold_timeout,
        })
    }
    fn post(&self, path: &str, body: Value, timeout: Duration) -> Result<Value, InferenceError> {
        let response = self
            .client
            .post(format!("{}{path}", self.endpoint))
            .timeout(timeout)
            .json(&body)
            .send()
            .map_err(|error| {
                if error.is_timeout() {
                    InferenceError::Timeout
                } else {
                    InferenceError::Failure
                }
            })?;
        if !response.status().is_success() {
            return Err(InferenceError::Failure);
        }
        // Bound responses without trusting Content-Length.
        use std::io::Read;
        let mut bytes = Vec::new();
        response
            .take(262145)
            .read_to_end(&mut bytes)
            .map_err(|error| {
                use std::error::Error;
                let mut cause: Option<&(dyn Error + 'static)> = Some(&error);
                while let Some(current) = cause {
                    if current
                        .downcast_ref::<reqwest::Error>()
                        .is_some_and(|e| e.is_timeout())
                    {
                        return InferenceError::Timeout;
                    }
                    cause = current.source();
                }
                if error.kind() == std::io::ErrorKind::TimedOut {
                    InferenceError::Timeout
                } else {
                    InferenceError::Failure
                }
            })?;
        if bytes.len() > 262144 {
            return Err(InferenceError::Malformed);
        }
        serde_json::from_slice(&bytes).map_err(|_| InferenceError::Malformed)
    }
}
impl Inference for Ollama {
    fn complete(
        &self,
        system: &str,
        user: &str,
        schema: Value,
        cold: bool,
    ) -> Result<Completion, InferenceError> {
        let started = Instant::now();
        let value=self.post("/api/chat",json!({"model":self.model,"stream":false,"think":false,"keep_alive":"600s","format":schema,"messages":[{"role":"system","content":system},{"role":"user","content":user}],"options":{"temperature":0.2,"num_ctx":4096,"num_predict":384}}),if cold {self.cold_timeout}else{self.warm_timeout})?;
        let content = value["message"]["content"]
            .as_str()
            .ok_or(InferenceError::Malformed)?;
        Ok(Completion {
            value: serde_json::from_str(content).map_err(|_| InferenceError::Malformed)?,
            timing: Timing {
                elapsed_ms: started.elapsed().as_millis(),
                load_ns: value["load_duration"].as_u64().unwrap_or(0),
                prompt_ns: value["prompt_eval_duration"].as_u64().unwrap_or(0),
                generation_ns: value["eval_duration"].as_u64().unwrap_or(0),
                generated_tokens: value["eval_count"].as_u64().unwrap_or(0),
            },
        })
    }
    fn retain(&self, idle: Option<Duration>) -> Result<(), String> {
        let keep = idle
            .map(|d| json!(format!("{}s", d.as_secs())))
            .unwrap_or(json!("600s"));
        self.post(
            "/api/generate",
            json!({"model":self.model,"stream":false,"keep_alive":keep,"options":{"temperature":0.2,"num_ctx":4096,"num_predict":384}}),
            Duration::from_secs(10),
        )
        .map(|_| ())
        .map_err(|_| "Ollama retention failed".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };
    fn fixture(body: Value, delay: Duration) -> (Ollama, thread::JoinHandle<Value>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let task = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut bytes = Vec::new();
            let mut buf = [0; 4096];
            loop {
                let n = socket.read(&mut buf).unwrap();
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
                                .and_then(|s| s.trim().parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if bytes.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let end = bytes.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
            let request = serde_json::from_slice(&bytes[end..]).unwrap();
            thread::sleep(delay);
            let body = body.to_string();
            let _ = write!(
                socket,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            request
        });
        (
            Ollama::new(
                &format!("http://{addr}"),
                "fixture",
                Duration::from_millis(40),
                Duration::from_secs(2),
            )
            .unwrap(),
            task,
        )
    }
    #[test]
    fn schema_thinking_retention_and_timing() {
        let (adapter, task) = fixture(
            json!({"message":{"content":"{\"answer\":\"12 °C\"}"},"load_duration":123,"eval_count":4}),
            Duration::ZERO,
        );
        let reply = adapter
            .complete("system", "user", json!({"type":"object"}), false)
            .unwrap();
        let req = task.join().unwrap();
        assert_eq!(req["think"], false);
        assert_eq!(req["stream"], false);
        assert_eq!(req["keep_alive"], "600s");
        assert_eq!(req["options"]["num_ctx"], 4096);
        assert_eq!(reply.timing.load_ns, 123);
        assert_eq!(reply.value["answer"], "12 °C");
        let (adapter, task) = fixture(json!({"done":true}), Duration::ZERO);
        adapter.retain(Some(Duration::from_secs(300))).unwrap();
        assert_eq!(task.join().unwrap()["keep_alive"], "300s");
    }
    #[test]
    fn warm_timeout_and_malformed_response_are_failures() {
        let (adapter, task) = fixture(json!({"message":{"content":"not json"}}), Duration::ZERO);
        assert_eq!(
            adapter.complete("s", "u", json!({}), false).unwrap_err(),
            InferenceError::Malformed
        );
        task.join().unwrap();
        let (adapter, task) = fixture(
            json!({"message":{"content":"{}"}}),
            Duration::from_millis(100),
        );
        assert_eq!(
            adapter.complete("s", "u", json!({}), false).unwrap_err(),
            InferenceError::Timeout
        );
        task.join().unwrap();
    }
}
