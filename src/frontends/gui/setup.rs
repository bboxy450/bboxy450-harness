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
        out.push((
            format!("setup/{slug}/details"),
            Action::SetupDetails(entry.agent),
        ));
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
fn centered(element: c::Element) -> c::Element {
    f::hstack(
        vec![
            f::fill(c::Element::Spacer),
            element.into(),
            f::fill(c::Element::Spacer),
        ],
        0.,
    )
}
pub fn element(
    snapshot: &Snapshot,
    ctx: ui::Context,
    width: f32,
    details: &std::collections::BTreeSet<Agent>,
    notices: &[&str],
) -> c::Element {
    let ink = f::ink(ctx);
    let panel_width = (width - 32.).clamp(208., 600.);
    let mut title = f::text("Choose your coding agent", 20., 600, ink.fg, false);
    title.align = c::Alignment::Center;
    let mut children = vec![
        centered(f::identity_mark(&ink, 0.5, 1., "setup/portrait")).into(),
        named("setup/title", c::Element::Text(title)).into(),
        centered(paragraph(
            "Installed agents appear automatically.",
            13.5,
            400,
            ink.dim,
            false,
        ))
        .into(),
    ];
    if let Some(error) = &snapshot.error {
        children.push(paragraph(error, 13.5, 400, ink.ask, false).into());
    }
    for notice in notices {
        children.push(paragraph(*notice, 12., 400, ink.ask, false).into());
    }
    for entry in &snapshot.entries {
        let slug = entry.agent.cli();
        let expanded = details.contains(&entry.agent);
        let install = snapshot.installs.get(&entry.agent);
        let running = install.is_some_and(|i| i.running);
        let available =
            entry.adapter && entry.profile && !(entry.agent == Agent::Pi && !entry.native);
        let action = if running {
            button(&format!("setup/{slug}/cancel"), "Cancel", ctx, false)
        } else if available {
            button(&format!("setup/{slug}/use"), "Use", ctx, false)
        } else if !entry.recipes.is_empty() {
            button(
                &format!("setup/{slug}/install/0"),
                if install.is_some() {
                    "Retry"
                } else {
                    "Install"
                },
                ctx,
                false,
            )
        } else {
            c::Element::Spacer
        };
        let mut card = vec![
            f::hstack(
                vec![
                    f::fill(paragraph(entry.agent.name(), 15., 600, ink.fg, false)),
                    action.into(),
                ],
                8.,
            )
            .into(),
            f::hstack(
                vec![
                    f::fill(paragraph(
                        entry.status(),
                        12.,
                        400,
                        if entry.adapter { ink.seat } else { ink.dim },
                        false,
                    )),
                    button(
                        &format!("setup/{slug}/details"),
                        if expanded { "Less" } else { "Details" },
                        ctx,
                        false,
                    )
                    .into(),
                ],
                8.,
            )
            .into(),
        ];
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
        if expanded {
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
            let detail = match entry.agent {
                Agent::Omp => {
                    "Native OMP ACP support. Authentication and model selection stay with OMP."
                }
                Agent::Pi => {
                    "Community pi-acp bridge; requires Pi CLI. MCP servers are not forwarded; configure Pi separately."
                }
                _ => {
                    "Authentication and model selection stay with your agent. Profiles remain editable; existing files are preserved."
                }
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
            if !running {
                for (i, recipe) in entry.recipes.iter().enumerate() {
                    card.push(paragraph(recipe.display(), 11., 400, ink.fg, true).into());
                    if i > 0 {
                        card.push(
                            button(
                                &format!("setup/{slug}/install/{i}"),
                                format!("Install with {:?}", recipe.manager),
                                ctx,
                                false,
                            )
                            .into(),
                        );
                    }
                }
                if entry.recipes.is_empty() && !entry.adapter {
                    card.push(paragraph("Use the official instructions. No supported package manager is available here.", 11., 400, ink.dim, false).into());
                }
            }
            card.push(paragraph("~/.bboxy450-harness/profiles", 11., 400, ink.dim, true).into());
        }
        children.push(
            named(
                format!("setup/{slug}/row"),
                f::frame(
                    f::Frame {
                        fill: ink.surface,
                        border: Some(f::alpha(ink.dim, 0.25)),
                        radius: [12.; 4],
                        padding: [8., 12., 8., 12.],
                        gap: 0.,
                        ..Default::default()
                    },
                    card,
                ),
            )
            .into(),
        );
    }
    children.push(centered(button("setup/close", "Back to conversation", ctx, false)).into());
    let mut panel = f::frame(
        f::Frame {
            gap: 8.,
            ..Default::default()
        },
        children,
    );
    if let c::Element::Box(b) = &mut panel {
        b.style.overrides.width = Some(ui::Length::Px(panel_width));
    }
    named(
        "setup",
        f::frame(
            f::Frame {
                padding: [24., 0., 24., 0.],
                ..Default::default()
            },
            vec![centered(named("setup/panel", panel)).into()],
        ),
    )
}
