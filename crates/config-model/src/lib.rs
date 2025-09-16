use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

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
    pub providers: BTreeMap<String, ProviderTemplate>,
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

fn args_contain(args: &[String], needle: &str) -> bool {
    args.iter().any(|a| a.contains(needle))
}

/// Validate providers templates for required placeholders per known provider.
pub fn validate_providers_config(cfg: &ProvidersConfig) -> Result<(), ConfigError> {
    let mut errors: Vec<String> = Vec::new();
    for (name, t) in &cfg.providers {
        if t.cmd.trim().is_empty() {
            errors.push(format!("providers.{name}.cmd must not be empty"));
        }
        let oneshot_has_prompt = args_contain(&t.oneshot_args, "{prompt}");
        let repl_has_system = args_contain(&t.repl_args, "{system_prompt}");
        let oneshot_has_session = args_contain(&t.oneshot_args, "{session_id}");
        let any_has_allowed = args_contain(&t.oneshot_args, "{allowed_tools}") || args_contain(&t.repl_args, "{allowed_tools}");
        let oneshot_has_chat = args_contain(&t.oneshot_args, "{chat_id}");
        let repl_has_chat = args_contain(&t.repl_args, "{chat_id}");

        match name.as_str() {
            // Claude Code expectations
            "claude" => {
                if !oneshot_has_prompt {
                    errors.push("providers.claude.oneshot_args must include {prompt}".into());
                }
                if !oneshot_has_session && !repl_has_system {
                    // session id is usually needed for reuse; tolerate if REPL will inject system prompt
                    errors.push("providers.claude: expected {session_id} in oneshot_args or a REPL flow".into());
                }
                if t.allowlist_flag.is_some() && !any_has_allowed {
                    errors.push("providers.claude: allowlist_flag set but {allowed_tools} placeholder missing in args".into());
                }
            }
            // Cursor Agent expectations
            k if k.starts_with("cursor") => {
                if !oneshot_has_prompt {
                    errors.push(format!("providers.{k}.oneshot_args must include {{prompt}}"));
                }
                if !oneshot_has_chat || !repl_has_chat {
                    errors.push(format!("providers.{k}: {{chat_id}} required in oneshot_args and repl_args"));
                }
            }
            // Gemini CLI expectations
            "gemini" => {
                if !oneshot_has_prompt {
                    errors.push("providers.gemini.oneshot_args must include {prompt}".into());
                }
                if !repl_has_system {
                    errors.push("providers.gemini.repl_args must include {system_prompt}".into());
                }
                if t.allowlist_flag.is_some() && !any_has_allowed {
                    errors.push("providers.gemini: allowlist_flag set but {allowed_tools} placeholder missing in args".into());
                }
            }
            _ => {
                // Unknown provider key: no strict validation
            }
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(ConfigError::Validation(errors.join("; "))) }
}

/// Validate a project config against providers config.
pub fn validate_project_config(project: &ProjectConfig, providers: &ProvidersConfig) -> Result<(), ConfigError> {
    let mut errors: Vec<String> = Vec::new();
    if project.schema_version != 1 { errors.push("project.schema_version must be 1".into()); }

    // Agent names must be unique and providers must exist
    let mut names = HashSet::new();
    for (idx, a) in project.agents.iter().enumerate() {
        if a.name.trim().is_empty() {
            errors.push(format!("agents[{idx}].name must not be empty"));
        }
        if !names.insert(a.name.clone()) {
            errors.push(format!("duplicate agent name: {}", a.name));
        }
        if !providers.providers.contains_key(&a.provider) {
            errors.push(format!("agents[{idx}].provider '{}' not found in providers.yaml", a.provider));
        }
        // allowed_tools policy: for claude/gemini must be non-empty
        match a.provider.as_str() {
            "claude" | "gemini" => {
                if a.allowed_tools.is_empty() {
                    errors.push(format!("agents[{idx}] (provider={}): allowed_tools must not be empty", a.provider));
                }
            }
            _ => {}
        }
        // system_prompt should not be empty
        if a.system_prompt.trim().is_empty() {
            errors.push(format!("agents[{idx}].system_prompt must not be empty"));
        }
    }

    // Group members must reference existing agent names
    for (gidx, g) in project.groups.iter().enumerate() {
        for m in &g.members {
            if !names.contains(m) {
                errors.push(format!("groups[{gidx}].members contains unknown agent name: {m}"));
            }
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(ConfigError::Validation(errors.join("; "))) }
}
