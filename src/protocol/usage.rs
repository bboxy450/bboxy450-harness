// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::*;
impl Usage {
    /// Only known numbers participate in sums; placeholders remain missing.
    pub fn numbers(&self) -> BTreeMap<String, u64> {
        let mut numbers = BTreeMap::new();
        for (key, value) in [
            ("input_tokens", Some(self.input_tokens)),
            ("output_tokens", Some(self.output_tokens)),
            ("cached_input_tokens", self.cached_input_tokens),
            (
                "cache_creation_input_tokens",
                self.cache_creation_input_tokens,
            ),
            ("reasoning_tokens", self.reasoning_tokens),
            ("cost_micros_usd", self.cost_micros_usd),
            ("context_used", self.context_used),
            ("context_window", self.context_window),
        ] {
            if let Some(value) = value
                && !self.unreported.iter().any(|s| s == key)
            {
                numbers.insert(key.into(), value);
            }
        }
        if let Some(cost) = &self.cost {
            numbers.insert("cost.amount_micros".into(), cost.amount_micros);
        }
        numbers
    }
    pub fn has_reported_numbers(&self) -> bool {
        !self.numbers().is_empty()
    }
    pub fn reported(&mut self, key: &str, present: bool) {
        self.source = Some(NumberSource::Reported);
        if matches!(key, "input_tokens" | "output_tokens") {
            self.unreported.retain(|s| s != key);
            if !present {
                self.unreported.push(key.into());
            }
        }
    }
    pub fn authored(mut self, scope: UsageScope, backend: Backend, model: Option<String>) -> Self {
        if self.source.is_none() {
            if self.input_tokens == 0 {
                self.unreported.push("input_tokens".into());
            }
            if self.output_tokens == 0 {
                self.unreported.push("output_tokens".into());
            }
            self.source = Some(NumberSource::Reported);
        }
        self.unreported.sort();
        self.unreported.dedup();
        self.scope = Some(scope);
        self.backend = Some(Box::new(backend));
        if self.model.is_none() {
            self.model = model;
        }
        if self.cost.is_none()
            && let Some(amount_micros) = self.cost_micros_usd
        {
            self.cost = Some(Box::new(Cost {
                amount_micros,
                currency: "USD".into(),
                source: NumberSource::Reported,
                price_source: Some("provider".into()),
            }));
        }
        if let Some(cost) = &self.cost
            && cost.currency == "USD"
        {
            self.cost_micros_usd = Some(cost.amount_micros);
        }
        self
    }
    /// A turn sum is derived; an incomplete required counter stays a placeholder.
    pub fn sum_calls(calls: &[Self]) -> Result<Option<Self>, &'static str> {
        let Some(first) = calls.first() else {
            return Ok(None);
        };
        fn sum(calls: &[Usage], key: &str) -> Result<Option<u64>, &'static str> {
            let mut total = 0u64;
            for call in calls {
                let Some(n) = call.numbers().get(key).copied() else {
                    return Ok(None);
                };
                total = total.checked_add(n).ok_or("usage total overflow")?;
            }
            Ok(Some(total))
        }
        let input = sum(calls, "input_tokens")?;
        let output = sum(calls, "output_tokens")?;
        let mut unreported = vec![];
        if input.is_none() {
            unreported.push("input_tokens".into());
        }
        if output.is_none() {
            unreported.push("output_tokens".into());
        }
        let cost = if calls.iter().all(|c| {
            c.cost.as_ref().map(|v| &v.currency) == first.cost.as_ref().map(|v| &v.currency)
        }) {
            sum(calls, "cost.amount_micros")?.and_then(|amount_micros| {
                first.cost.as_ref().map(|c| Cost {
                    amount_micros,
                    currency: c.currency.clone(),
                    source: NumberSource::Derived,
                    price_source: if calls.iter().all(|u| {
                        u.cost.as_ref().and_then(|c| c.price_source.as_ref())
                            == c.price_source.as_ref()
                    }) {
                        c.price_source.clone()
                    } else {
                        None
                    },
                })
            })
        } else {
            None
        };
        let total = Self {
            input_tokens: input.unwrap_or(0),
            output_tokens: output.unwrap_or(0),
            unreported,
            cached_input_tokens: sum(calls, "cached_input_tokens")?,
            cache_creation_input_tokens: sum(calls, "cache_creation_input_tokens")?,
            reasoning_tokens: sum(calls, "reasoning_tokens")?,
            cost_micros_usd: sum(calls, "cost_micros_usd")?,
            cost: cost.map(Box::new),
            scope: Some(UsageScope::Turn),
            backend: first.backend.clone(),
            source: Some(NumberSource::Derived),
            model: if calls.iter().all(|c| c.model == first.model) {
                first.model.clone()
            } else {
                None
            },
            ..Default::default()
        };
        Ok(total.has_reported_numbers().then_some(total))
    }
}
