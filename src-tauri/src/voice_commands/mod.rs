// Voice commands module - command matching and execution

pub mod registry;

use registry::{ActionType, CommandDefinition, CommandRegistry, RegistryError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// State wrapper for the command registry
pub struct VoiceCommandsState {
    pub registry: Mutex<CommandRegistry>,
}

impl VoiceCommandsState {
    pub fn new() -> Result<Self, RegistryError> {
        let mut registry = CommandRegistry::with_default_path()?;
        registry.load()?;
        Ok(Self {
            registry: Mutex::new(registry),
        })
    }
}

/// DTO for command definition (for Tauri serialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDto {
    pub id: String,
    pub trigger: String,
    pub action_type: String,
    pub parameters: HashMap<String, String>,
    pub enabled: bool,
}

impl From<&CommandDefinition> for CommandDto {
    fn from(cmd: &CommandDefinition) -> Self {
        let action_type = match cmd.action_type {
            ActionType::OpenApp => "open_app",
            ActionType::TypeText => "type_text",
            ActionType::SystemControl => "system_control",
            ActionType::Workflow => "workflow",
            ActionType::Custom => "custom",
        };
        Self {
            id: cmd.id.to_string(),
            trigger: cmd.trigger.clone(),
            action_type: action_type.to_string(),
            parameters: cmd.parameters.clone(),
            enabled: cmd.enabled,
        }
    }
}

/// Input for adding a new command
#[derive(Debug, Clone, Deserialize)]
pub struct AddCommandInput {
    pub trigger: String,
    pub action_type: String,
    pub parameters: HashMap<String, String>,
    pub enabled: bool,
}

fn parse_action_type(s: &str) -> Result<ActionType, String> {
    match s {
        "open_app" => Ok(ActionType::OpenApp),
        "type_text" => Ok(ActionType::TypeText),
        "system_control" => Ok(ActionType::SystemControl),
        "workflow" => Ok(ActionType::Workflow),
        "custom" => Ok(ActionType::Custom),
        _ => Err(format!("Unknown action type: {}", s)),
    }
}

/// Get all registered commands
#[tauri::command]
pub fn get_commands(
    state: tauri::State<'_, VoiceCommandsState>,
) -> Result<Vec<CommandDto>, String> {
    let registry = state
        .registry
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    Ok(registry.list().iter().map(|c| CommandDto::from(*c)).collect())
}

/// Add a new command
#[tauri::command]
pub fn add_command(
    state: tauri::State<'_, VoiceCommandsState>,
    input: AddCommandInput,
) -> Result<CommandDto, String> {
    let action_type = parse_action_type(&input.action_type)?;
    let cmd = CommandDefinition {
        id: Uuid::new_v4(),
        trigger: input.trigger,
        action_type,
        parameters: input.parameters,
        enabled: input.enabled,
    };

    let mut registry = state
        .registry
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    registry.add(cmd.clone()).map_err(|e| e.to_string())?;
    Ok(CommandDto::from(&cmd))
}

/// Remove a command by ID
#[tauri::command]
pub fn remove_command(
    state: tauri::State<'_, VoiceCommandsState>,
    id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| format!("Invalid UUID: {}", e))?;
    let mut registry = state
        .registry
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    registry.delete(uuid).map_err(|e| e.to_string())
}
