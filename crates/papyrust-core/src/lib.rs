//! Core types and logic for papyrust.
//!
//! This crate is the contract between the CLI and the renderers. It owns:
//!
//! - the `book.toml` schema and parser ([`config`]),
//! - the in-memory Book intermediate representation (`ir`, coming in M1),
//! - the on-disk project layout walker (`project`, coming in M1),
//! - the Markdown → IR conversion (`parse`, coming in M1),
//! - and validation of a project before rendering (`validate`, coming in M1).
//!
//! Renderer crates depend on this crate; this crate depends on no other
//! workspace crate.

pub mod config;
pub mod ir;
pub mod parse;
pub mod project;
pub mod validate;
