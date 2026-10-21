use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize)]
pub struct Capability {
    pub id: String,
    pub description: String,
    pub inputs: Value,
    pub progress: String,
    pub command: Option<Command>,
}
/// One slash command binding the literal remainder to at most one string field.
#[derive(Clone, Debug, Serialize)]
pub struct Command {
    pub name: String,
    pub argument: Option<String>,
}
pub const BUILTIN_COMMANDS: &[&str] = &["help", "exit", "quit", "profile"];
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
pub fn validate_command(capability: &Capability) -> Result<(), String> {
    let Some(command) = &capability.command else {
        return Ok(());
    };
    if !valid_name(&command.name) || BUILTIN_COMMANDS.contains(&command.name.as_str()) {
        return Err("Invalid or reserved module command".into());
    }
    let properties = capability.inputs["properties"]
        .as_object()
        .ok_or("Invalid schema")?;
    match &command.argument {
        Some(field) if properties.len() == 1 && properties.contains_key(field) => Ok(()),
        None if properties.is_empty() => Ok(()),
        _ => Err(
            "Command must bind the capability's single string field or have empty inputs".into(),
        ),
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    NeedsInput,
    Error,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub title: String,
    pub url: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleResult {
    pub outcome: Outcome,
    pub fallback: String,
    /// Canonical facts when fallback adds a presentation label. None uses fallback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factual_text: Option<String>,
    /// Display-ready literal fields select deterministic authored presentation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_fields: Option<std::collections::BTreeMap<String, String>>,
    pub data: Option<Value>,
    pub sources: Vec<Source>,
}
impl ModuleResult {
    pub fn message(outcome: Outcome, text: impl Into<String>) -> Self {
        Self {
            outcome,
            fallback: text.into(),
            factual_text: None,
            template_fields: None,
            data: None,
            sources: vec![],
        }
    }
}
pub trait Module: Send + Sync {
    fn capability(&self) -> Capability;
    fn execute(&self, arguments: &Value) -> ModuleResult;
}
/// Intentionally small schema subset, rejecting unsupported schemas rather than guessing.
pub fn validate_schema(schema: &Value) -> Result<(), String> {
    let object = schema
        .as_object()
        .ok_or("Capability schema must be an object")?;
    if object
        .keys()
        .any(|k| !["type", "properties", "required", "additionalProperties"].contains(&k.as_str()))
        || schema["type"] != "object"
        || schema["additionalProperties"] != false
    {
        return Err("Unsupported capability schema".into());
    }
    let properties = schema["properties"]
        .as_object()
        .ok_or("Invalid capability properties")?;
    let required = schema["required"]
        .as_array()
        .ok_or("Invalid required fields")?;
    let mut seen = std::collections::BTreeSet::new();
    for field in required {
        let name = field.as_str().ok_or("Invalid required field")?;
        if !properties.contains_key(name) || !seen.insert(name) {
            return Err("Invalid or duplicate required field".into());
        }
    }
    // Validate absent optional properties too: no constraint is silently ignored.
    for rule in properties.values() {
        let rule_object = rule.as_object().ok_or("Invalid property schema")?;
        if rule_object
            .keys()
            .any(|k| !["type", "minLength", "maxLength"].contains(&k.as_str()))
            || rule["type"] != "string"
        {
            return Err("Unsupported property schema".into());
        }
        for bound in ["minLength", "maxLength"] {
            if let Some(value) = rule.get(bound) {
                value
                    .as_u64()
                    .ok_or("Length bound must be a nonnegative integer")?;
            }
        }
        let min = rule["minLength"].as_u64().unwrap_or(0);
        let max = rule["maxLength"].as_u64().unwrap_or(4096);
        if min > max {
            return Err("Invalid property length bounds".into());
        }
    }
    Ok(())
}
pub fn validate(schema: &Value, args: &Value) -> Result<(), String> {
    validate_schema(schema)?;
    let object = args.as_object().ok_or("Arguments must be an object")?;
    let properties = schema["properties"]
        .as_object()
        .ok_or("Invalid capability schema")?;
    for required in schema["required"]
        .as_array()
        .ok_or("Invalid required fields")?
    {
        if !object.contains_key(required.as_str().ok_or("Invalid required field")?) {
            return Err("Missing required information".into());
        }
    }
    for (key, value) in object {
        let rule = properties.get(key).ok_or("Unknown argument")?;
        let text = value.as_str().ok_or("Argument must be text")?;
        let length = text.chars().count() as u64;
        let min = rule["minLength"].as_u64().unwrap_or(0);
        if length < min
            || length > rule["maxLength"].as_u64().unwrap_or(4096)
            || (min > 0 && text.trim().is_empty())
        {
            return Err("Argument length is outside allowed bounds".into());
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn old_results_remain_compatible_and_optional_fields_round_trip() {
        let mut result: ModuleResult = serde_json::from_value(serde_json::json!({
            "outcome":"success", "fallback":"Original.", "data":{"value":12}, "sources":[]
        }))
        .unwrap();
        assert!(result.template_fields.is_none());
        assert!(
            serde_json::to_value(&result)
                .unwrap()
                .get("template_fields")
                .is_none()
        );
        result.template_fields = Some(std::collections::BTreeMap::from([(
            "value".into(),
            "12 knots".into(),
        )]));
        let decoded: ModuleResult =
            serde_json::from_value(serde_json::to_value(&result).unwrap()).unwrap();
        assert_eq!(decoded.template_fields, result.template_fields);
        assert_eq!(decoded.data, result.data);
        assert_eq!(decoded.fallback, result.fallback);
    }

    #[test]
    fn schema_rejects_missing_unknown_wrong_and_empty() {
        let s = json!({"type":"object","properties":{"q":{"type":"string","minLength":1,"maxLength":4}},"required":["q"],"additionalProperties":false});
        for a in [
            json!({}),
            json!({"q":1}),
            json!({"q":" "}),
            json!({"q":"abcde"}),
            json!({"q":"ok","x":"x"}),
        ] {
            assert!(validate(&s, &a).is_err());
        }
        assert!(validate(&s, &json!({"q":"yes"})).is_ok());
    }
    #[test]
    fn empty_schema_accepts_only_empty_object() {
        let schema =
            json!({"type":"object","properties":{},"required":[],"additionalProperties":false});
        assert!(validate_schema(&schema).is_ok());
        assert!(validate(&schema, &json!({})).is_ok());
        for args in [json!({"path":"."}), json!([]), json!(null), json!("{}")] {
            assert!(validate(&schema, &args).is_err());
        }
    }
    #[test]
    fn raw_length_and_whitespace_are_separate_constraints() {
        let schema = json!({"type":"object","properties":{"q":{"type":"string","minLength":1,"maxLength":4}},"required":["q"],"additionalProperties":false});
        assert!(validate(&schema, &json!({"q":"     okay     "})).is_err());
        assert!(validate(&schema, &json!({"q":"    "})).is_err());
        assert!(validate(&schema, &json!({"q":" éé "})).is_ok());
    }
    #[test]
    fn schemas_fail_closed_even_for_absent_optional_properties() {
        let schema = json!({"type":"object","properties":{"q":{"type":"string"}},"required":[],"additionalProperties":false});
        let mut examples = Vec::new();
        for rule in [
            json!({"type":"string","enum":["allowed"]}),
            json!({"type":"string","minLength":"1"}),
            json!({"type":"string","maxLength":-1}),
            json!({"type":"string","maxLength":1.5}),
            json!({"type":"string","minLength":5,"maxLength":4}),
            json!({"type":"number"}),
            json!({"type":"string","pattern":"allowed"}),
        ] {
            let mut s = schema.clone();
            s["properties"]["q"] = rule;
            examples.push(s);
        }
        let mut s = schema.clone();
        s["required"] = json!(["missing"]);
        examples.push(s);
        let mut s = schema.clone();
        s["required"] = json!(["q", "q"]);
        examples.push(s);
        let mut s = schema.clone();
        s["unevaluatedProperties"] = json!(false);
        examples.push(s);
        for s in examples {
            assert!(validate_schema(&s).is_err(), "{s}");
            assert!(validate(&s, &json!({})).is_err(), "{s}");
            assert!(validate(&s, &json!({"q":"forbidden"})).is_err(), "{s}");
        }
    }
}
