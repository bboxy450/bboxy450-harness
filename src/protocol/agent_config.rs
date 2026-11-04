// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Safe session configuration projection. Only advertised model/thought selectors cross this boundary.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentChoice {
    pub value: String,
    pub name: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentOption {
    pub id: String,
    pub name: String,
    pub category: String,
    pub current: String,
    pub choices: Vec<AgentChoice>,
    pub legacy: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentConfig {
    /// Unique child incarnation, independent of session/profile names (including A→B→A).
    pub instance: OpId,
    pub revision: u64,
    pub options: Vec<AgentOption>,
    pub pending: Option<OpId>,
    pub status: String,
}
impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            instance: OpId::new(),
            revision: 0,
            options: vec![],
            pending: None,
            status: "Agent configuration unavailable · session not negotiated".into(),
        }
    }
}
impl AgentConfig {
    pub fn replace(&mut self, result: &Value) {
        let mut options = vec![];
        for raw in result
            .get("configOptions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let id = raw.get("id").and_then(Value::as_str).unwrap_or("");
            let category = raw.get("category").and_then(Value::as_str).unwrap_or(id);
            if !matches!(category, "model" | "thought_level" | "reasoning_effort")
                || raw.get("type").and_then(Value::as_str) != Some("select")
                || id.is_empty()
            {
                continue;
            }
            let Some(current) = raw.get("currentValue").and_then(Value::as_str) else {
                continue;
            };
            let mut choices = vec![];
            for entry in raw
                .get("options")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                for v in entry
                    .get("options")
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or(std::slice::from_ref(entry))
                {
                    if let (Some(value), Some(name)) = (
                        v.get("value").and_then(Value::as_str),
                        v.get("name").and_then(Value::as_str),
                    ) {
                        choices.push(AgentChoice {
                            value: value.into(),
                            name: name.into(),
                        });
                    }
                }
            }
            options.push(AgentOption {
                id: id.into(),
                name: raw.get("name").and_then(Value::as_str).unwrap_or(id).into(),
                category: if category == "reasoning_effort" {
                    "thought_level"
                } else {
                    category
                }
                .into(),
                current: current.into(),
                choices,
                legacy: false,
            });
        }
        // Old ACP adapters advertised session/new.models, with session/set_model.
        if !options.iter().any(|o| o.category == "model")
            && let Some(models) = result.get("models")
            && let Some(current) = models.get("currentModelId").and_then(Value::as_str)
        {
            let choices = models
                .get("availableModels")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|m| {
                    Some(AgentChoice {
                        value: m.get("modelId")?.as_str()?.into(),
                        name: m.get("name")?.as_str()?.into(),
                    })
                })
                .collect();
            options.insert(
                0,
                AgentOption {
                    id: "model".into(),
                    name: "Model".into(),
                    category: "model".into(),
                    current: current.into(),
                    choices,
                    legacy: true,
                },
            );
        }
        self.options = options;
        self.revision += 1;
    }
    pub fn admits(&self, instance: OpId, revision: u64, id: &str, value: &str) -> bool {
        self.instance == instance
            && self.revision == revision
            && self.pending.is_none()
            && self
                .options
                .iter()
                .any(|o| o.id == id && o.choices.iter().any(|c| c.value == value))
    }
}
