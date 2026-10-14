//! Markdown linting and fixing rules for MkDocs.

mod cli;
mod diff;
mod files;
mod markdown;
mod model;
mod rules;
mod shell;
mod syntax;

pub use cli::run_command;
pub use markdown::{check_markdown, fix_markdown};
pub use model::{FixResult, Issue};
pub use shell::split_command_line;

#[cfg(test)]
mod regression_tests;
