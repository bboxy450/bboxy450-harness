// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Automation: a hosted decision model answers an agent's multiple-choice
//! question for the operator. It picks one named option or leaves the question
//! to the human; it never executes, plans or keeps state.
//!
//! The wire is TypeSafe's System One (`POST /v1/systemone`, a `choice` question):
//! https://docs.typesafe.ai/api/system-one.md
//!
//! Rules, in order:
//! - only single-select fields with 2..=6 options are answered; an optional
//!   free-text field is left empty, a required one makes the question the human's;
//! - a question that mentions credentials, publishing or anything irreversible
//!   is the human's, and the model is never called;
//! - the model is asked twice with the options in opposite order and must agree
//!   with itself, at or above the confidence floor, on a real option;
//! - any failure, timeout or doubt leaves the question waiting for the human.
use crate::redact::{Secret, Set};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{path::Path, time::Duration};
use tokio_util::sync::CancellationToken;

/// A JSON [`Spec`] in the environment overrides the file.
pub const ENV: &str = "BBOXY450_HARNESS_DECISION_MODEL";
/// The machine's decision model, beside `profiles/` and `sessions/`.
pub const FILE: &str = "decision-model.json";
/// The option the model picks to hand the question back.
const ASK_HUMAN: &str = "ask_human";
/// Jev's `confidence` is `(p_max - 1/n) / (1 - 1/n)`: 0.6 is p = 0.8 of two.
const CONFIDENCE: f64 = 0.6;
/// Both calls together. A human would take far longer; a hang must not.
const DEADLINE: Duration = Duration::from_secs(5);
const DEFAULT_MODEL: &str = "jev-1.14.0";
const MAX_OPTIONS: usize = 6;
const MAX_CONTEXT: usize = 4096;
const MAX_REPLY: usize = 64 * 1024;
/// Crude on purpose: a match can only send a question to the human.
const HUMAN_ONLY: &[&str] = &[
    "credential",
    "password",
    "passphrase",
    "secret",
    "token",
    "api key",
    "login",
    "log in",
    "sign in",
    "publish",
    "release",
    "push",
    "merge",
    "deploy",
    "pay",
    "spend",
    "purchase",
    "delete",
    "remove",
    "force",
    "reset",
    "drop",
    "wipe",
    "irreversible",
];
const INSTRUCTIONS: &str = "A coding agent asks its operator this question and offers these options. Pick the option the operator would most plausibly choose, given the operator's latest request. Prefer the option that keeps the work inside what was asked. Pick ask_human when the answer depends on a preference you cannot infer, or when a wrong pick would be costly.";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    url: String,
    #[serde(default)]
    model: Option<String>,
    /// The key, or `!command` whose output is the key (as a profile's `api_key`).
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    api_key_env: Option<String>,
}

/// A configured decision model.
pub struct Model {
    url: String,
    model: String,
    key: Secret,
    client: reqwest::Client,
}

/// What automation did with one question.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// An ACP elicitation response, ready for the agent.
    Answer {
        answer: Value,
        model: String,
        confidence: f64,
    },
    /// The question stays with the human, for this reason.
    Human(String),
}

impl Model {
    /// `Ok(None)` when no decision model is configured; an error names what is wrong.
    pub async fn load(root: &Path, redactor: &Set) -> Result<Option<Self>, String> {
        let text = match std::env::var(ENV) {
            Ok(text) => text,
            Err(_) => match tokio::fs::read_to_string(root.join(FILE)).await {
                Ok(text) => text,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(format!("{FILE}: {error}")),
            },
        };
        let spec: Spec = serde_json::from_str(&text).map_err(|e| format!("{FILE}: {e}"))?;
        crate::provider::profile::validate_endpoint(&spec.url)
            .map_err(|e| format!("{FILE}: {}", e.message))?;
        let key = match (&spec.api_key, &spec.api_key_env) {
            (Some(source), _) => match source.strip_prefix('!') {
                Some(cmd) => {
                    crate::provider::profile::resolve_command(cmd, CancellationToken::new())
                        .await
                        .map_err(|e| format!("{FILE}: {}", e.message))?
                }
                None => source.clone(),
            },
            (None, Some(name)) => {
                std::env::var(name).map_err(|_| format!("{FILE}: {name} is not set"))?
            }
            (None, None) => return Err(format!("{FILE}: no api_key or api_key_env")),
        };
        if key.trim().is_empty() {
            return Err(format!("{FILE}: empty key"));
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(DEADLINE)
            .build()
            .map_err(|_| "decision model: HTTP client unavailable".to_owned())?;
        Ok(Some(Self {
            url: spec.url,
            model: spec.model.unwrap_or_else(|| DEFAULT_MODEL.into()),
            key: Secret::new(key.trim().into(), redactor),
            client,
        }))
    }

    /// Answer `message` over `schema`, or say why the human must.
    /// `context` is the operator's latest request; it is capped here.
    pub async fn decide(&self, message: &str, schema: &Value, context: &str) -> Outcome {
        let fields = match eligible(message, schema) {
            Ok(fields) => fields,
            Err(reason) => return Outcome::Human(reason.into()),
        };
        let state = json!({
            "agent_question": message,
            "operator_latest_request": cap(context),
        });
        let ask = |reverse| self.ask(&state, &fields, reverse);
        let both = async { tokio::join!(ask(false), ask(true)) };
        let (first, second) = match tokio::time::timeout(DEADLINE, both).await {
            Ok((Ok(first), Ok(second))) => (first, second),
            Ok((Err(reason), _)) | Ok((_, Err(reason))) => return Outcome::Human(reason),
            Err(_) => return Outcome::Human("the decision model timed out".into()),
        };
        let mut content = serde_json::Map::new();
        let mut lowest = 1.0f64;
        for field in &fields {
            let (Some(a), Some(b)) = (pick(&first, field), pick(&second, field)) else {
                return Outcome::Human("the decision model's reply was not a choice".into());
            };
            if a.0 != b.0 {
                return Outcome::Human("the decision model did not agree with itself".into());
            }
            if a.0 == ASK_HUMAN {
                return Outcome::Human("the decision model asked for you".into());
            }
            let confidence = a.1.min(b.1);
            if confidence.is_nan() || confidence < CONFIDENCE {
                return Outcome::Human(format!(
                    "the decision model was not confident ({confidence:.2})"
                ));
            }
            lowest = lowest.min(confidence);
            content.insert(field.key.clone(), Value::String(a.0));
        }
        Outcome::Answer {
            answer: json!({"action": "accept", "content": content}),
            model: first
                .get("model")
                .and_then(Value::as_str)
                .unwrap_or(&self.model)
                .to_owned(),
            confidence: lowest,
        }
    }

    async fn ask(
        &self,
        state: &Value,
        fields: &[QuestionField],
        reverse: bool,
    ) -> Result<Value, String> {
        // serde_json sorts object keys; the option order on the wire is the point
        // of the second call, so the body is assembled in order by hand.
        let questions = fields
            .iter()
            .map(|field| {
                let mut options: Vec<String> = field
                    .options
                    .iter()
                    .zip(&field.details)
                    .map(|((value, label), detail)| {
                        let text = match detail {
                            Some(detail) if label != value => format!("{label}: {detail}"),
                            Some(detail) => detail.clone(),
                            None => label.clone(),
                        };
                        format!("{}:{}", Value::from(value.as_str()), Value::from(text))
                    })
                    .collect();
                if reverse {
                    options.reverse();
                }
                options.push(format!(
                    "{}:{}",
                    Value::from(ASK_HUMAN),
                    Value::from("Leave this question to the human operator")
                ));
                let asked = [field.title.as_deref(), field.description.as_deref()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" ");
                format!(
                    "{}:{{\"type\":\"choice\",\"instructions\":{},\"criteria\":{{{}}}}}",
                    Value::from(field.key.as_str()),
                    Value::from(format!("{INSTRUCTIONS} {asked}").trim()),
                    options.join(",")
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let body = format!(
            "{{\"model\":{},\"state\":{state},\"questions\":{{{questions}}}}}",
            Value::from(self.model.as_str())
        );
        let reply = self
            .client
            .post(&self.url)
            .bearer_auth(self.key.expose())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|_| "the decision model was unreachable".to_owned())?;
        let status = reply.status();
        if !status.is_success() {
            return Err(format!("the decision model refused ({})", status.as_u16()));
        }
        let bytes = reply
            .bytes()
            .await
            .map_err(|_| "the decision model's reply was cut short".to_owned())?;
        if bytes.len() > MAX_REPLY {
            return Err("the decision model's reply was too large".into());
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| "the decision model's reply was not JSON".to_owned())
    }
}

/// The fields automation may answer, or why the question is the human's.
pub fn eligible(message: &str, schema: &Value) -> Result<Vec<QuestionField>, &'static str> {
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut fields = question_fields(schema);
    // An optional free-text field (AskUserQuestion's "Other") is left empty.
    if fields
        .iter()
        .any(|f| f.options.is_empty() && required.contains(&f.key.as_str()))
    {
        return Err("it needs free text");
    }
    fields.retain(|f| !f.options.is_empty());
    if fields.is_empty() {
        return Err("it has no options");
    }
    let mut text = message.to_lowercase();
    for field in &fields {
        if field.multi {
            return Err("it allows several answers");
        }
        if !(2..=MAX_OPTIONS).contains(&field.options.len()) {
            return Err("it does not have two to six options");
        }
        if field.options.iter().any(|(value, _)| value == ASK_HUMAN) {
            return Err("an option is reserved");
        }
        for part in [&field.title, &field.description].into_iter().flatten() {
            text.push(' ');
            text.push_str(&part.to_lowercase());
        }
        for ((value, label), detail) in field.options.iter().zip(&field.details) {
            for part in [Some(value), Some(label), detail.as_ref()]
                .into_iter()
                .flatten()
            {
                text.push(' ');
                text.push_str(&part.to_lowercase());
            }
        }
    }
    if HUMAN_ONLY.iter().any(|word| text.contains(word)) {
        return Err("it touches credentials, publishing or something irreversible");
    }
    Ok(fields)
}

/// The model's choice and confidence for one field, when the reply names a real option.
fn pick(reply: &Value, field: &QuestionField) -> Option<(String, f64)> {
    let answer = reply.get("answers")?.get(&field.key)?;
    let choice = answer.get("choice")?.as_str()?;
    let confidence = answer.get("confidence")?.as_f64()?;
    (choice == ASK_HUMAN || field.options.iter().any(|(value, _)| value == choice))
        .then(|| (choice.to_owned(), confidence))
}

fn cap(text: &str) -> &str {
    let mut end = text.len().min(MAX_CONTEXT);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// One field of a Question's form: a select (`oneOf`, `anyOf` or `enum`; an
/// array type is multi-select) or, with no options, free text.
#[derive(Debug)]
pub struct QuestionField {
    pub key: String,
    pub title: Option<String>,
    pub description: Option<String>,
    /// (value sent back, label shown)
    pub options: Vec<(String, String)>,
    /// Each option's own description, where the schema gives one.
    pub details: Vec<Option<String>>,
    pub multi: bool,
}
pub fn question_fields(schema: &serde_json::Value) -> Vec<QuestionField> {
    let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) else {
        return vec![];
    };
    properties
        .iter()
        .map(|(key, p)| {
            let multi = p.get("type").and_then(|t| t.as_str()) == Some("array");
            let source = if multi {
                p.get("items").unwrap_or(p)
            } else {
                p
            };
            let (options, details) = ["oneOf", "anyOf", "enum"]
                .iter()
                .find_map(|k| source.get(*k).and_then(|v| v.as_array()))
                .into_iter()
                .flatten()
                .filter_map(|o| match o {
                    serde_json::Value::String(s) => Some(((s.clone(), s.clone()), None)),
                    serde_json::Value::Object(_) => {
                        let value = o.get("const")?.as_str()?.to_owned();
                        let label = o
                            .get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or(&value)
                            .to_owned();
                        let detail = o
                            .get("description")
                            .and_then(|t| t.as_str())
                            .map(str::to_owned);
                        Some(((value, label), detail))
                    }
                    _ => None,
                })
                .unzip();
            let text = |k: &str| p.get(k).and_then(|v| v.as_str()).map(str::to_owned);
            QuestionField {
                key: key.clone(),
                title: text("title"),
                description: text("description"),
                options,
                details,
                multi,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema(extra: Value) -> Value {
        let mut schema = json!({"type": "object", "properties": {
            "q": {"type": "string", "title": "DB", "oneOf": [
                {"const": "Postgres", "title": "Postgres", "description": "a server"},
                {"const": "SQLite", "title": "SQLite"}]},
            "q_custom": {"type": "string", "title": "Other"}}});
        for (key, value) in extra.as_object().unwrap() {
            schema[key] = value.clone();
        }
        schema
    }

    #[test]
    fn a_select_with_an_optional_free_text_companion_is_eligible() {
        let fields = eligible("Which database?", &schema(json!({}))).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "q");
        assert_eq!(fields[0].details, vec![Some("a server".to_owned()), None]);
    }

    #[test]
    fn the_human_keeps_free_text_multi_select_long_lists_and_risky_words() {
        let required = schema(json!({"required": ["q_custom"]}));
        assert_eq!(
            eligible("Which?", &required).unwrap_err(),
            "it needs free text"
        );
        let text_only = json!({"properties": {"a": {"type": "string"}}});
        assert_eq!(
            eligible("Name?", &text_only).unwrap_err(),
            "it has no options"
        );
        let multi = json!({"properties": {"a": {"type": "array", "items": {"enum": ["x", "y"]}}}});
        assert_eq!(
            eligible("Which?", &multi).unwrap_err(),
            "it allows several answers"
        );
        let many = json!({"properties": {"a": {"enum": ["1", "2", "3", "4", "5", "6", "7"]}}});
        assert_eq!(
            eligible("Which?", &many).unwrap_err(),
            "it does not have two to six options"
        );
        let one = json!({"properties": {"a": {"enum": ["1"]}}});
        assert_eq!(
            eligible("Which?", &one).unwrap_err(),
            "it does not have two to six options"
        );
        let reserved = json!({"properties": {"a": {"enum": ["x", "ask_human"]}}});
        assert_eq!(
            eligible("Which?", &reserved).unwrap_err(),
            "an option is reserved"
        );
        for risky in [
            "Push to main now?",
            "Which API key?",
            "Delete the old tables?",
        ] {
            assert!(
                eligible(risky, &schema(json!({})))
                    .unwrap_err()
                    .contains("irreversible")
            );
        }
        let option = json!({"properties": {"a": {"enum": ["keep", "force overwrite"]}}});
        assert!(eligible("How?", &option).is_err());
    }

    #[test]
    fn a_reply_must_name_a_real_option_with_a_confidence() {
        let fields = eligible("Which database?", &schema(json!({}))).unwrap();
        let reply = |choice: &str| json!({"answers": {"q": {"choice": choice, "confidence": 0.9}}});
        assert_eq!(
            pick(&reply("SQLite"), &fields[0]),
            Some(("SQLite".into(), 0.9))
        );
        assert_eq!(
            pick(&reply(ASK_HUMAN), &fields[0]),
            Some((ASK_HUMAN.into(), 0.9))
        );
        assert_eq!(pick(&reply("MySQL"), &fields[0]), None);
        assert_eq!(
            pick(&json!({"answers": {"q": {"choice": "SQLite"}}}), &fields[0]),
            None
        );
        assert_eq!(pick(&json!({}), &fields[0]), None);
    }

    #[test]
    fn context_is_capped_on_a_character_boundary() {
        let long = "é".repeat(MAX_CONTEXT);
        assert!(cap(&long).len() <= MAX_CONTEXT);
        assert!(long.is_char_boundary(cap(&long).len()));
    }
}
