//! Skilo - CLI tool for Agent Skills development.
//!
//! This crate provides tools for creating, validating, and formatting
//! [Agent Skills](https://agentskills.io/specification).

pub mod agent;
pub mod cache;
pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod git;
pub mod lang;
pub mod output;
pub mod scope;
pub mod skill;
pub mod templates;

pub use error::{Result, SkiloError};
