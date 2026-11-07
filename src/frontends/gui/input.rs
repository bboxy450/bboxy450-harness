// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Draft/session ownership of asynchronous intake. Both native hosts call these methods.
use super::super::attachments::{self, Attachment, Intake};
use super::*;
use std::{path::PathBuf, sync::mpsc};
use ui::text_edit::{Command as Edit, Completion, CompletionPopover};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Owner {
    session: Option<SessionId>,
    epoch: Epoch,
    generation: u64,
    draft: u64,
}
struct Reply {
    owner: Owner,
    index: usize,
    value: Result<Value, String>,
}
enum Value {
    Candidates(Completion, Vec<String>),
    Attached(Vec<Attachment>),
}
pub(super) struct Sent {
    pub(super) owner: Owner,
    pub(super) op: OpId,
    pub(super) draft: composer::Submission,
    pub(super) attachments: Vec<Attachment>,
    pub(super) pending_at_send: Arc<im::OrdMap<OpId, super::super::bridge::PendingPrompt>>,
}
pub(super) struct Inputs {
    pub(super) owner: Owner,
    root: Option<PathBuf>,
    waker: Option<Arc<dyn bbt_rt::Waker>>,
    tx: mpsc::Sender<Reply>,
    rx: mpsc::Receiver<Reply>,
    query: Option<Completion>,
    pub status: Option<String>,
    intake_owner: Option<Owner>,
    loading: usize,
    pub(super) attachments: Vec<Attachment>,
    pub(super) sent: Vec<Sent>,
    jobs: [Option<tokio::task::JoinHandle<()>>; 2],
    intake_jobs: Vec<tokio::task::JoinHandle<()>>,
    pub(super) observed_text: String,
}
impl Default for Inputs {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            owner: Owner {
                session: None,
                epoch: Epoch(0),
                generation: 0,
                draft: 0,
            },
            root: None,
            waker: None,
            tx,
            rx,
            query: None,
            status: None,
            intake_owner: None,
            loading: 0,
            attachments: vec![],
            sent: vec![],
            jobs: [None, None],
            intake_jobs: vec![],
            observed_text: String::new(),
        }
    }
}
impl Drop for Inputs {
    fn drop(&mut self) {
        for job in self.jobs.iter().flatten().chain(&self.intake_jobs) {
            job.abort();
        }
    }
}
impl Inputs {
    pub(super) fn changed(&mut self) {
        self.owner.draft += 1;
        self.query = None;
        self.status = None;
        self.loading = 0;
    }
    fn spawn(
        &mut self,
        index: usize,
        job: impl FnOnce() -> Result<Value, String> + Send + 'static,
    ) {
        self.intake_jobs.retain(|j| !j.is_finished());
        if let Some(old) = self.jobs[index].take() {
            if index == 0 {
                old.abort();
            } else {
                self.intake_jobs.push(old);
            }
        }
        let owner = self.owner.clone();
        let tx = self.tx.clone();
        let wake = self.waker.clone();
        self.jobs[index] = Some(bbt_rt::handle().spawn(async move {
            let value = crate::runtime::blocking(job).await;
            if tx
                .send(Reply {
                    owner,
                    index,
                    value,
                })
                .is_ok()
                && let Some(wake) = wake
            {
                wake.wake();
            }
        }));
    }
}
impl HarnessView {
    /// Install the host's wake target and current admitted session before processing input.
    pub fn update_inputs(&mut self, bridge: &Bridge) {
        self.inputs.waker = Some(bridge.input_waker());
        self.poll_inputs(&bridge.state);
    }
    pub(super) fn poll_inputs(&mut self, state: &FrameState) {
        // Reconnecting faces must not infer delivery or replace attachment ownership
        // from an incomplete replay. Sending remains fenced by Bridge readiness.
        if !state.ready && self.inputs.owner.session.is_some() {
            return;
        }
        if self.inputs.observed_text != self.composer.view.text {
            self.inputs.changed();
            self.inputs.observed_text = self.composer.view.text.clone();
        }
        let scope = (state.transcript.session, state.transcript.epoch);
        if scope != (self.inputs.owner.session, self.inputs.owner.epoch) {
            self.inputs.owner.session = scope.0;
            self.inputs.owner.epoch = scope.1;
            self.inputs.owner.generation += 1;
            self.inputs.changed();
            self.completion = None;
            // Preserve the draft, but its attachments cannot be sent into a replacement.
            if !self.inputs.attachments.is_empty() {
                self.inputs.status = Some(
                    "Attachments belong to the previous session; remove them and select again"
                        .into(),
                );
            }
        }
        self.inputs.root = state.transcript.meta.as_ref().map(|m| m.cwd.clone());
        // Each submitted draft owns its own custody. A busy turn or an unknown
        // delivery never serializes editing/sending subsequent drafts.
        let mut pending = Vec::new();
        for sent in std::mem::take(&mut self.inputs.sent) {
            let same = sent.owner.session == scope.0
                && sent.owner.epoch == scope.1
                && sent.owner.generation == self.inputs.owner.generation;
            let durable = same
                && !Arc::ptr_eq(&sent.pending_at_send, &state.pending_prompts)
                && !state.pending_prompts.contains_key(&sent.op);
            let refused = same
                && state
                    .pending_prompts
                    .get(&sent.op)
                    .is_some_and(|p| p.status.starts_with("Rejected"));
            if durable {
                continue;
            }
            if refused && self.composer.view.text.is_empty() && self.composer.view.attach.is_empty()
            {
                let mut view = self.composer.view.clone();
                view.text = sent.draft.text;
                view.attach = sent.draft.attach;
                self.composer = super::super::editor::Input::new(view);
                self.inputs.observed_text = self.composer.view.text.clone();
                self.inputs.attachments = sent.attachments;
                self.inputs.intake_owner = Some(self.inputs.owner.clone());
                self.inputs.changed();
                self.notice = Some("Message not sent; attachment draft restored".into());
                self.invalidate();
            } else {
                pending.push(sent);
            }
        }
        self.inputs.sent = pending;
        while let Ok(reply) = self.inputs.rx.try_recv() {
            if reply.owner != self.inputs.owner {
                continue;
            }
            match reply.value {
                Ok(Value::Candidates(request, rows))
                    if self.file_request().as_ref() == Some(&request) =>
                {
                    if self.inputs.loading == 0 {
                        self.inputs.status = rows
                            .is_empty()
                            .then(|| "No matching workspace files".into());
                    }
                    self.completion =
                        (!rows.is_empty()).then(|| CompletionPopover::new(request, rows));
                }
                Ok(Value::Attached(rows)) => {
                    self.inputs.loading = self.inputs.loading.saturating_sub(1);
                    if self.inputs.attachments.len() + rows.len() > 8 {
                        self.inputs.status = Some("At most eight attachments per draft".into());
                        continue;
                    }
                    self.inputs.intake_owner = Some(self.inputs.owner.clone());
                    for a in rows {
                        let title = a.label.clone();
                        // File-style chips avoid mutating the editor from an async callback.
                        self.composer
                            .handle(composer::Command::Attach(composer::Attachment {
                                kind: "file".into(),
                                akind: if a.preview.is_some() { "image" } else { "file" }.into(),
                                title,
                                meta: "Ready".into(),
                                id: a.id.clone(),
                                path: a.id.clone(),
                                ..Default::default()
                            }));
                        if self.composer.view.text.is_empty() {
                            self.composer.handle(composer::Command::Edit(Edit::Insert(
                                if a.preview.is_some() {
                                    format!("[Image #{}]", self.inputs.attachments.len() + 1)
                                } else {
                                    format!("@{}", a.label)
                                },
                            )));
                        }
                        self.inputs.observed_text = self.composer.view.text.clone();
                        self.inputs.attachments.push(a);
                    }
                    self.inputs.status =
                        (self.inputs.loading > 0).then(|| "Reading attachment…".into());
                    // Leave the draft generation stable for a simultaneously issued directory result.
                }
                Ok(_) => {}
                Err(error) => {
                    if reply.index == 1 {
                        self.inputs.loading = self.inputs.loading.saturating_sub(1);
                    }
                    if reply.index == 1 || self.inputs.loading == 0 {
                        self.inputs.status = Some(error);
                    }
                }
            }
            self.invalidate();
        }
        self.refresh_files();
    }
    fn file_request(&self) -> Option<Completion> {
        if self.composer.composing() {
            return None;
        }
        let text = &self.composer.view.text;
        let edit = self.composer.view.edit.as_ref()?;
        if edit.select.is_some() {
            return None;
        }
        let caret = edit.caret?;
        let end = ui::components::utf16_byte_index(text, caret)?;
        let before = &text[..end];
        let at = before.rfind('@')?;
        if at > 0 && !before[..at].ends_with(char::is_whitespace) {
            return None;
        }
        let query = &before[at + 1..];
        if query.contains('\n') || query.ends_with(' ') {
            return None;
        }
        Some(Completion {
            trigger: '@',
            query: query.into(),
            range: [before[..at].encode_utf16().count(), caret],
        })
    }
    pub(super) fn refresh_files(&mut self) {
        let Some(request) = self.file_request() else {
            return;
        };
        if self.inputs.query.as_ref() == Some(&request) {
            return;
        }
        self.inputs.query = Some(request.clone());
        self.completion = None;
        let Some(root) = self
            .inputs
            .root
            .clone()
            .filter(|_| self.inputs.owner.session.is_some())
        else {
            self.inputs.status = Some("Workspace not admitted yet".into());
            return;
        };
        self.inputs.status = Some("Loading workspace files…".into());
        self.inputs.spawn(0, move || {
            attachments::candidates(&root, &request.query)
                .map(|rows| Value::Candidates(request, rows))
        });
    }
    /// Snapshot sources are supplied by the native host; tests inject private bytes.
    pub fn intake(&mut self, source: Intake, bridge: &Bridge) {
        self.update_inputs(bridge);
        if !bridge.state.ready || self.inputs.owner.session.is_none() {
            self.inputs.status = Some("Session not ready; attachment was not read".into());
            return;
        }
        if self.composer.composing() {
            self.inputs.status = Some("Finish text composition before attaching".into());
            return;
        }
        if self.inputs.attachments.len() >= 8
            || self.inputs.loading >= 8
            || self
                .inputs
                .intake_jobs
                .iter()
                .filter(|j| !j.is_finished())
                .count()
                >= 7
        {
            self.inputs.status = Some("At most eight attachments per draft".into());
            return;
        }
        let Some(root) = self.inputs.root.clone() else {
            self.inputs.status = Some("Workspace unavailable".into());
            return;
        };
        self.inputs.status = Some("Reading attachment…".into());
        self.inputs.loading += 1;
        self.inputs.spawn(1, move || {
            let rows = match source {
                Intake::EncodedImage { bytes, png } => {
                    vec![attachments::image(png(&bytes)?, "Pasted image".into())?]
                }
                Intake::Png(bytes) => vec![attachments::image(bytes, "Pasted image".into())?],
                Intake::Files(paths) => {
                    if paths.len() > 8 {
                        return Err("At most eight attachments per draft".into());
                    }
                    paths
                        .iter()
                        .map(|p| attachments::read_file(&root, p))
                        .collect::<Result<Vec<_>, _>>()?
                }
            };
            Ok(Value::Attached(rows))
        });
        self.invalidate();
    }
    pub fn attachment_status(&self) -> Option<&str> {
        self.inputs.status.as_deref()
    }
    pub fn attachment_images(&self) -> Vec<Arc<bbt_render::chrome::icons::Image>> {
        self.inputs
            .attachments
            .iter()
            .filter_map(|a| a.preview.clone())
            .collect()
    }
    pub fn pending_attachment_count(&self) -> usize {
        self.inputs.sent.iter().map(|s| s.attachments.len()).sum()
    }
    pub fn attachment_count(&self) -> usize {
        self.inputs.attachments.len()
    }
    pub(super) fn choose_file(&mut self, request: Completion, label: String, bridge: &Bridge) {
        if self.file_request().as_ref() != Some(&request) {
            return;
        }
        let Some(root) = self.inputs.root.clone() else {
            return;
        };
        self.composer
            .handle(composer::Command::Edit(Edit::SetCaret {
                at: request.range[0],
                extend: false,
            }));
        self.composer
            .handle(composer::Command::Edit(Edit::SetCaret {
                at: request.range[1],
                extend: true,
            }));
        self.edit(Edit::Insert(if label.ends_with('/') {
            label.clone()
        } else {
            format!("{label} ")
        }));
        self.completion = None;
        if !label.ends_with('/') {
            self.intake(
                Intake::Files(vec![root.join(label.trim_start_matches('@'))]),
                bridge,
            );
        }
    }
    pub(super) fn attachment_blocks(
        &self,
        sent: &composer::Submission,
    ) -> Result<Vec<InputBlock>, String> {
        if self.inputs.loading > 0 {
            return Err("Attachment is still loading".into());
        }
        if !sent.attach.is_empty() && self.inputs.attachments.len() != sent.attach.len() {
            return Err("Attachment unavailable".into());
        }
        if self.inputs.sent.len() >= 64 {
            return Err("Attachment delivery queue is full; draft retained".into());
        }
        if self
            .inputs
            .attachments
            .iter()
            .any(|a| !sent.attach.iter().any(|x| x.id == a.id))
        {
            return Err("Attachment selection changed".into());
        }
        if !sent.attach.is_empty()
            && self.inputs.intake_owner.as_ref().is_none_or(|o| {
                o.session != self.inputs.owner.session
                    || o.epoch != self.inputs.owner.epoch
                    || o.generation != self.inputs.owner.generation
            })
        {
            return Err(
                "Attachments belong to a previous session; remove and select them again".into(),
            );
        }
        let mut blocks = vec![];
        if !sent.text.trim().is_empty() {
            blocks.push(InputBlock::Text {
                text: sent.text.clone(),
            });
        }
        blocks.extend(self.inputs.attachments.iter().map(|a| a.block.clone()));
        // UTF-8 content remains in the same UserTurn and therefore cannot exceed
        // the existing Unix envelope bound or cause partial multi-prompt delivery.
        if serde_json::to_vec(&blocks)
            .map_err(|e| e.to_string())?
            .len()
            > 200 * 1024
        {
            return Err("Combined text attachments exceed 200 KiB; remove a file".into());
        }
        Ok(blocks)
    }
    pub(super) fn remove_attachment(&mut self, index: usize) {
        if index < self.inputs.attachments.len() {
            self.inputs.attachments.remove(index);
            self.composer
                .handle(composer::Command::RemoveAttachment(index));
            self.inputs.changed();
            self.invalidate();
        }
    }
    pub(super) fn file_menu(
        &self,
        p: &CompletionPopover,
        ink: &super::super::face::Ink,
    ) -> c::Element {
        use super::super::face::{Frame, fixed, frame, line};
        let selected = p.choices.selected().unwrap_or(0);
        let start = selected.saturating_sub(7);
        let mut rows = vec![fixed(line("Workspace files", 12., 600, ink.dim, false))];
        for (i, c) in p.choices.options.iter().enumerate().skip(start).take(8) {
            rows.push(fixed(named(
                format!("completion/{i}"),
                line(
                    &c.label,
                    12.,
                    if i == selected { 700 } else { 400 },
                    ink.fg,
                    false,
                ),
            )));
        }
        rows.push(fixed(line(
            "↑ ↓ choose · Enter / Tab select",
            11.,
            400,
            ink.dim,
            false,
        )));
        frame(
            Frame {
                padding: [8.; 4],
                gap: 8.,
                fill: ink.bg,
                ..Default::default()
            },
            rows,
        )
    }
    pub(super) fn attachment_panel(&self, composer: c::Element, width: f32) -> c::Element {
        use super::super::face::{self as f, Frame, fixed, frame, line};
        if self.inputs.attachments.is_empty() && self.inputs.status.is_none() {
            return composer;
        }
        let ink = f::ink(context());
        // One row of compact chips, as a terminal shows `[Image #1]`: a small
        // thumbnail, the label and a × that removes it.
        let mut chips = vec![];
        for (i, a) in self.inputs.attachments.iter().enumerate() {
            let mut chip = vec![];
            let label = if let Some(image) = &a.preview {
                chip.push(fixed(c::Element::Image {
                    asset: image.name.clone(),
                    size: ui::Length::Px(20.),
                    radius: ui::Length::Px(4.),
                }));
                if a.label == "Pasted image" {
                    format!("Image #{}", i + 1)
                } else {
                    a.label.clone()
                }
            } else {
                a.label.clone()
            };
            chip.push(fixed(line(&label, 11.5, 500, ink.fg, false)));
            chip.push(fixed(named(
                format!("attachment/remove/{i}"),
                frame(
                    Frame {
                        padding: [0., 2., 0., 2.],
                        ..Default::default()
                    },
                    vec![fixed(line("×", 13., 500, ink.dim, false))],
                ),
            )));
            chips.push(fixed(frame(
                Frame {
                    row: true,
                    gap: 5.,
                    fill: f::alpha(ink.fg, 0.06),
                    border: Some(f::alpha(ink.fg, 0.10)),
                    radius: [7.; 4],
                    padding: [2., 5., 2., 3.],
                    ..Default::default()
                },
                chip,
            )));
        }
        let mut rows = vec![];
        if !chips.is_empty() {
            rows.push(fixed(frame(
                Frame {
                    row: true,
                    gap: 6.,
                    ..Default::default()
                },
                chips,
            )));
        }
        if let Some(status) = &self.inputs.status {
            rows.push(fixed(line(status, 11.5, 400, ink.dim, false)));
        }
        let strip = frame(
            Frame {
                padding: [2., 12., 4., 12.],
                gap: 3.,
                ..Default::default()
            },
            rows,
        );
        let _ = width;
        frame(Frame::default(), vec![fixed(strip), fixed(composer)])
    }
}
