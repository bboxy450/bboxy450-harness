// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{protocol::Effect, provider::profile::Unattended};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EffectClass {
    ReadOnly,
    Write,
    Exec,
    Network,
}
impl EffectClass {
    pub fn effect(&self) -> Option<Effect> {
        match self {
            Self::ReadOnly => Some(Effect::Read),
            Self::Write => Some(Effect::Write),
            Self::Exec => Some(Effect::Exec),
            Self::Network => Some(Effect::Network),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleDecision {
    Allow,
    Ask,
    Deny,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub tool: String,
    pub effect: Option<EffectClass>,
    pub decision: RuleDecision,
}
#[derive(Clone, Debug)]
pub struct Action {
    pub tool: String,
    pub effect: EffectClass,
    pub generation: u64,
}
#[derive(Clone, Debug)]
pub enum Decision {
    Allow { rule: String },
    Ask,
    Deny { reason: String },
}
#[derive(Clone, Debug)]
pub struct Policy {
    pub rules: Vec<Rule>,
    pub on_unattended: Unattended,
}
impl Policy {
    /// Labels complement the independent descriptor capability. This profile
    /// alone is never an isolation proof and does not alter ordinary/Yolo policy.
    pub fn dream_trial() -> Self {
        Self {
            on_unattended: Unattended::Deny,
            rules: vec![
                Rule {
                    id: "dream-trial-deny".into(),
                    tool: "*".into(),
                    effect: None,
                    decision: RuleDecision::Deny,
                },
                Rule {
                    id: "dream-trial-read".into(),
                    tool: "read".into(),
                    effect: Some(EffectClass::ReadOnly),
                    decision: RuleDecision::Allow,
                },
                Rule {
                    id: "dream-trial-write".into(),
                    tool: "write".into(),
                    effect: Some(EffectClass::Write),
                    decision: RuleDecision::Allow,
                },
            ],
        }
    }
    pub fn new(on_unattended: Unattended) -> Self {
        Self {
            rules: vec![],
            on_unattended,
        }
    }
    pub fn allow_all(on_unattended: Unattended) -> Self {
        Self {
            rules: vec![Rule {
                id: "profile-yolo".into(),
                tool: "*".into(),
                effect: None,
                decision: RuleDecision::Allow,
            }],
            on_unattended,
        }
    }
    pub fn decide(&self, action: &Action) -> Decision {
        for r in self.rules.iter().rev() {
            if (r.tool == "*" || r.tool == action.tool)
                && r.effect.as_ref().is_none_or(|e| e == &action.effect)
            {
                return match r.decision {
                    RuleDecision::Allow => Decision::Allow { rule: r.id.clone() },
                    RuleDecision::Ask => Decision::Ask,
                    RuleDecision::Deny => Decision::Deny {
                        reason: format!("denied by rule {}", r.id),
                    },
                };
            }
        }
        if action.effect == EffectClass::ReadOnly {
            Decision::Allow {
                rule: "read-only".into(),
            }
        } else {
            Decision::Ask
        }
    }
    pub fn always_allow(&mut self, tool: &str, effect: EffectClass) {
        self.rules.push(Rule {
            id: format!("allow-always-{}", self.rules.len()),
            tool: tool.into(),
            effect: Some(effect),
            decision: RuleDecision::Allow,
        });
    }
}
#[derive(Clone, Debug)]
pub struct Permit {
    pub(crate) tool: String,
    pub(crate) generation: u64,
}
impl Permit {
    pub(crate) fn approval(tool: String, generation: u64) -> Self {
        Self { tool, generation }
    }
}
impl Policy {
    pub fn permit(&self, action: &Action) -> Option<Permit> {
        matches!(self.decide(action), Decision::Allow { .. }).then(|| Permit {
            tool: action.tool.clone(),
            generation: action.generation,
        })
    }
}
