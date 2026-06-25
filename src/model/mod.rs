//! Domain models for NOM, embedded contracts, and stats.
//!
// `model::json` is a single shared helper crate-local module (see
// `consolidate-json-helpers`); each model file pulls its whole surface via a
// `use crate::model::json::*;` glob so the per-file import list tracks the
// shared helper set automatically. The glob is intentional and scoped to this
// module tree.
#![allow(clippy::wildcard_imports)]

pub mod embedded;
pub(crate) mod json;
pub mod nom;
pub mod stats;
