// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! One setup face for standalone and embedded hosts, using the existing palette.
use super::{face as f, view::Action};
use crate::onboarding::{Agent, Snapshot};
use bbt_ui::{
    self as ui, components as c,
    modules::seat_row::named,
    widgets::{Button, ButtonKind, ButtonState},
};

fn paragraph(
    value: impl Into<String>,
    size: f32,
    weight: u16,
    color: f::Rgba,
    mono: bool,
) -> c::Element {
    let mut text = f::text(value, size, weight, color, mono);
    let mut runs = vec![];
    for run in text.runs {
        for word in run.value.split_inclusive(char::is_whitespace) {
            let chars: Vec<_> = word.chars().collect();
            for chunk in chars.chunks(16) {
                runs.push(c::TextRun {
                    value: chunk.iter().collect(),
                    style: run.style,
                });
            }
        }
    }
    text.runs = runs;
    c::Element::Text(text)
}
fn button(key: &str, text: impl Into<String>, ctx: ui::Context, disabled: bool) -> c::Element {
    named(
        key,
        Button {
            key: None,
            icon: None,
            label: text.into(),
            kind: ButtonKind::Ghost,
            state: if disabled {
                ButtonState::Disabled
            } else {
                ButtonState::Idle
            },
        }
        .element(ctx)
        .expect("existing button style"),
    )
}
pub fn actions(snapshot: &Snapshot) -> Vec<(String, Action)> {
    let mut out = vec![("setup/close".into(), Action::Setup(false))];
    for entry in &snapshot.entries {
        let slug = entry.agent.cli();
        let running = snapshot
            .installs
            .get(&entry.agent)
            .is_some_and(|i| i.running);
        if running {
            out.push((
                format!("setup/{slug}/cancel"),
                Action::CancelInstall(entry.agent),
            ));
        } else {
            for (i, recipe) in entry.recipes.iter().enumerate() {
                out.push((
                    format!("setup/{slug}/install/{i}"),
                    Action::InstallAdapter(recipe.clone()),
                ));
            }
        }
        if entry.adapter && entry.profile && !(entry.agent == Agent::Pi && !entry.native) {
            out.push((
                format!("setup/{slug}/use"),
                Action::SwitchProfile(slug.into()),
            ));
        }
    }
    out
}
pub fn element(snapshot: &Snapshot, ctx: ui::Context) -> c::Element {
    let ink = f::ink(ctx);
    let mut children = vec![
        f::identity_mark(&ink, 0.5, 1., "setup").into(),
        paragraph("Choose your coding agent", 20., 600, ink.fg, false).into(),
        paragraph(
            "Installed agents are discovered automatically. Profiles stay editable on disk.",
            13.5,
            400,
            ink.dim,
            false,
        )
        .into(),
        paragraph(
            "~/.bboxy450-harness/profiles · existing files are always preserved",
            11.,
            400,
            ink.dim,
            true,
        )
        .into(),
        button("setup/close", "Back to conversation", ctx, false).into(),
    ];
    if let Some(error) = &snapshot.error {
        children.push(paragraph(error, 13.5, 400, ink.ask, false).into());
    }
    for entry in &snapshot.entries {
        let slug = entry.agent.cli();
        let install = snapshot.installs.get(&entry.agent);
        let running = install.is_some_and(|i| i.running);
        let mut card = vec![
            paragraph(entry.agent.name(), 15., 600, ink.fg, false).into(),
            paragraph(
                entry.status(),
                13.5,
                400,
                if entry.adapter { ink.seat } else { ink.dim },
                false,
            )
            .into(),
        ];
        card.push(
            paragraph(
                if entry.native {
                    "Native CLI installed on PATH"
                } else {
                    "Native CLI not found on PATH"
                },
                11.,
                400,
                ink.dim,
                false,
            )
            .into(),
        );
        let detail = if entry.agent == Agent::Omp {
            "OMP documents ACP client filesystem and terminal tools that this client does not provide. Use its official instructions to configure a compatible custom profile."
        } else if entry.agent == Agent::Pi {
            "Pi uses the community pi-acp bridge and requires the Pi CLI. This adapter does not forward MCP servers; configure Pi separately."
        } else {
            "Authentication and model selection remain with your agent and profile."
        };
        card.push(paragraph(detail, 11., 400, ink.dim, false).into());
        card.push(
            paragraph(
                format!("Instructions: {}", entry.agent.instructions()),
                11.,
                400,
                ink.dim,
                true,
            )
            .into(),
        );
        if let Some(install) = install {
            card.push(
                paragraph(
                    if running {
                        "Installing…"
                    } else {
                        install.result.as_deref().unwrap_or("Ready")
                    },
                    13.5,
                    500,
                    ink.seat,
                    false,
                )
                .into(),
            );
            let output = install
                .output
                .lines()
                .rev()
                .take(4)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|line| line.chars().take(100).collect::<String>())
                .collect::<Vec<_>>()
                .join("\n");
            if !output.is_empty() {
                card.push(paragraph(output, 11., 400, ink.dim, true).into());
            }
        }
        if running {
            card.push(
                button(
                    &format!("setup/{slug}/cancel"),
                    "Cancel installation",
                    ctx,
                    false,
                )
                .into(),
            );
        } else {
            for (i, recipe) in entry.recipes.iter().enumerate() {
                card.push(paragraph(recipe.display(), 11., 400, ink.fg, true).into());
                card.push(
                    button(
                        &format!("setup/{slug}/install/{i}"),
                        if install.is_some() {
                            "Retry installation"
                        } else {
                            "Install"
                        },
                        ctx,
                        false,
                    )
                    .into(),
                );
            }
            if entry.recipes.is_empty() && !entry.adapter {
                card.push(paragraph("Use the official instructions. No supported package manager recipe is available here; managers are never installed automatically.",11.,400,ink.dim,false).into());
            }
        }
        if entry.adapter && entry.profile && !(entry.agent == Agent::Pi && !entry.native) {
            card.push(button(&format!("setup/{slug}/use"), "Use this agent", ctx, false).into());
        }
        children.push(
            f::frame(
                f::Frame {
                    fill: ink.surface,
                    border: Some(f::alpha(ink.dim, 0.25)),
                    radius: [12.; 4],
                    padding: [16.; 4],
                    gap: 8.,
                    ..Default::default()
                },
                card,
            )
            .into(),
        );
    }
    named(
        "setup",
        f::frame(
            f::Frame {
                padding: [24.; 4],
                gap: 16.,
                ..Default::default()
            },
            children,
        ),
    )
}
