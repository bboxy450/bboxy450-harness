// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! Host-owned upgraded editor, with toolkit composer presentation/submission.
use bbt_ui::{
    modules::composer::{Command, Composer, Controller, Field, Outcome, Submission},
    text_edit::{Command as Edit, Completion, CompletionPopover, Editor, Motion},
};
use std::ops::{Deref, DerefMut};
pub struct Input {
    inner: Controller,
    editor: Editor,
}
impl Deref for Input {
    type Target = Controller;
    fn deref(&self) -> &Controller {
        &self.inner
    }
}
impl DerefMut for Input {
    fn deref_mut(&mut self) -> &mut Controller {
        &mut self.inner
    }
}
impl Input {
    pub fn new(view: Composer) -> Self {
        let editor = Editor::new(&view.text);
        Self {
            inner: Controller::new(view),
            editor,
        }
    }
    pub fn composing(&self) -> bool {
        self.editor.composing() || self.inner.composing()
    }
    pub fn completion(&self) -> Option<Completion> {
        self.editor.completion(&self.inner.view.text)
    }
    pub fn popover(&self, profiles: &[String]) -> Option<CompletionPopover> {
        let request = self.completion()?;
        let choices = if request.trigger == '/' {
            ["/interrupt".to_owned(), "/stop".into(), "/profiles".into()]
                .into_iter()
                .chain(profiles.iter().map(|p| format!("/profile {p}")))
                .filter(|s| s[1..].starts_with(&request.query))
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        (!choices.is_empty()).then(|| CompletionPopover::new(request, choices))
    }
    pub fn vertical(&mut self, up: bool) {
        let selection = self.editor.selection();
        let at =
            bbt_ui::components::utf16_byte_index(&self.inner.view.text, selection[1]).unwrap_or(0);
        let first = selection[0] == selection[1] && !self.inner.view.text[..at].contains('\n');
        let mut changed = false;
        if first {
            changed = self.editor.handle(
                &mut self.inner.view.text,
                if up {
                    Edit::HistoryPrevious
                } else {
                    Edit::HistoryNext
                },
            );
        }
        if !changed {
            self.editor.handle(
                &mut self.inner.view.text,
                Edit::Move {
                    motion: if up { Motion::Up } else { Motion::Down },
                    extend: false,
                },
            );
        }
        self.sync();
    }
    pub fn handle(&mut self, command: Command) -> Outcome {
        if matches!(command, Command::Blur | Command::Focus(Field::Address)) {
            self.editor
                .handle(&mut self.inner.view.text, Edit::CancelPreedit);
        }
        let result = match command {
            Command::Edit(edit) if self.inner.view.field == Field::Body => {
                let changed = self.editor.handle(&mut self.inner.view.text, edit);
                self.inner.view.body_scroll = None;
                if changed {
                    Outcome::Changed
                } else {
                    Outcome::Local
                }
            }
            Command::Send if self.editor.composing() => Outcome::Local,
            command => self.inner.handle(command),
        };
        self.sync();
        result
    }
    pub fn stored(&mut self, sent: &Submission) {
        self.editor.remember_input(&sent.text);
        self.inner.stored(sent);
        if self.inner.view.text.is_empty() {
            self.editor.reset("");
        }
        self.sync();
    }
    fn sync(&mut self) {
        self.inner.view.edit = (self.inner.view.focus && self.inner.view.field == Field::Body)
            .then(|| self.editor.edit());
    }
}
