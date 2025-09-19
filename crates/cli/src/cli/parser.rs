//! CLI parsing and execution logic

use super::commands::*;
use crate::commands::*;

impl Cli {
    /// Execute the parsed CLI command
    pub fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        match self.cmd {
            Commands::Init { config_dir, force, skip_db } => 
                run_init(config_dir.as_deref(), force, skip_db),
            Commands::Config { cmd } => match cmd {
                ConfigCmd::Validate { project_file, providers_file, format } => {
                    run_config_validate(project_file.as_deref(), providers_file.as_deref(), format)
                }
                ConfigCmd::Init { dir, force } => run_config_init(dir.as_deref(), force),
            },
            Commands::Doctor { format, ndjson_sample, snapshot } => 
                run_doctor(format, ndjson_sample.as_deref(), snapshot.as_deref()),
            Commands::Db { cmd } => match cmd {
                DbCmd::Init { db_path } => run_db_init(db_path.as_deref()),
                DbCmd::ProjectAdd { name, db_path } => run_project_add(&name, db_path.as_deref()),
                DbCmd::AgentAdd { project, name, role, provider, model, allowed_tool, system_prompt, db_path } =>
                    run_agent_add(&project, &name, &role, &provider, &model, &allowed_tool, &system_prompt, db_path.as_deref()),
            },
            Commands::Send { project_file, providers_file, to, message, session_id, chat_id, timeout_ms, format, progress } => {
                run_send(project_file.as_deref(), providers_file.as_deref(), &to, &message, session_id.as_deref(), chat_id.as_deref(), timeout_ms, format, progress)
            },
            Commands::Session { cmd } => match cmd {
                SessionCmd::Start { project_file, providers_file, agent } =>
                    run_session_start(project_file.as_deref(), providers_file.as_deref(), &agent),
                SessionCmd::List { project_file, project, agent, provider, format } =>
                    run_session_list(project_file.as_deref(), project.as_deref(), agent.as_deref(), provider.as_deref(), format),
                SessionCmd::Resume { conversation_id, timeout_ms } =>
                    run_session_resume(&conversation_id, timeout_ms),
                SessionCmd::Cleanup { project_file, dry_run, format } =>
                    run_session_cleanup(project_file.as_deref(), dry_run, format),
            },
            Commands::Agent { cmd } => match cmd {
                AgentCmd::Run { project_file, providers_file, project, agent, role, provider, model, workdir, no_logs, timeout_ms } =>
                    run_agent_run(project_file.as_deref(), providers_file.as_deref(), project.as_deref(), &agent, role.as_deref(), provider.as_deref(), model.as_deref(), workdir.as_deref(), no_logs, timeout_ms),
                AgentCmd::Attach { project_file, project, agent, timeout_ms } =>
                    run_agent_attach(project_file.as_deref(), project.as_deref(), &agent, timeout_ms),
                AgentCmd::Stop { project_file, project, agent, timeout_ms } =>
                    run_agent_stop(project_file.as_deref(), project.as_deref(), &agent, timeout_ms),
            },
        }
    }
}
