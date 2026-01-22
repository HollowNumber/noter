//! Template system for dynamic document generation
//!
//! This module provides a comprehensive template system with support for
//! dynamic content, variants, and configuration-driven behaviour.

pub mod builder;
pub mod config;
mod constants;
pub mod context;
pub mod discovery;
pub mod engine;
pub mod fetcher;
pub mod validation;
