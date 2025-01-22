mod planning;

pub use planning::PlanningAgent;

use crate::inferences::{LanguageModel, Message, Role};
use crate::services::SessionState;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// Reads the agent's base instruction from the file system.
fn read_instruction(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .expect("Failed to read the base instruction")
}
