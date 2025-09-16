use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub schema_version: u32,
    pub project: String,
    pub agents: Vec<AgentConfig>,
    #[serde(default)]
    pub groups: Vec<GroupConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    pub name: String,
    pub role: String,
    pub provider: String,
    pub model: String,
    pub allowed_tools: Vec<String>,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GroupConfig {
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProvidersConfig {
    pub schema_version: u32,
    pub providers: std::collections::BTreeMap<String, ProviderTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProviderTemplate {
    pub cmd: String,
    #[serde(default)]
    pub oneshot_args: Vec<String>,
    #[serde(default)]
    pub repl_args: Vec<String>,
    #[serde(default)]
    pub create_chat_args: Option<Vec<String>>, // cursor-agent
    #[serde(default)]
    pub allowlist_flag: Option<String>,       // claude/gemini
    #[serde(default)]
    pub forbid_flags: Option<Vec<String>>,    // cursor --force, etc.
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid yaml: {0}")]
    InvalidYaml(String),
    #[error("validation error: {0}")]
    Validation(String),
}

pub fn parse_project_yaml(yaml: &str) -> Result<ProjectConfig, ConfigError> {
    serde_yaml::from_str::<ProjectConfig>(yaml)
        .map_err(|e| ConfigError::InvalidYaml(e.to_string()))
}

pub fn parse_providers_yaml(yaml: &str) -> Result<ProvidersConfig, ConfigError> {
    serde_yaml::from_str::<ProvidersConfig>(yaml)
        .map_err(|e| ConfigError::InvalidYaml(e.to_string()))
}

pub fn json_schema_project() -> schemars::Schema {
    schemars::schema_for!(ProjectConfig)
}

pub fn json_schema_providers() -> schemars::Schema {
    schemars::schema_for!(ProvidersConfig)
}
