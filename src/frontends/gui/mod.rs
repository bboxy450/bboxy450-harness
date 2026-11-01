// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! GUI face. The model and gateway bridge also build without a UI backend.
#[cfg(feature = "embedded")]
pub mod assets;
pub mod bridge;
pub mod chat_scroll;
pub mod model;
pub mod options;
#[cfg(feature = "embedded")]
pub mod smoke;
#[cfg(feature = "embedded")]
pub mod view;
#[cfg(feature = "standalone")]
pub mod window;
