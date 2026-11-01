// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

#![allow(clippy::disallowed_methods)] // IO modules; GUI frame modules deny this lint.

//! One core, one event log, and protocol consumers for every face.

pub mod protocol;

pub mod process;
pub mod redact;
pub mod runtime;
pub mod store;

pub mod provider;

pub mod policy;

pub mod tools;

pub mod gateway;

pub mod engine;

pub mod frontends;

pub mod context;
