//! Integration tests for database operations

use crate::commands::db::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_commands_smoke() {
        // Use a temp DB path
        let tmp = tempfile::tempdir().unwrap();
        let dbp = tmp.path().join("multi-agents.sqlite3");
        let dbs = dbp.to_string_lossy().to_string();

        // init
        run_db_init(Some(&dbs)).expect("db init");
        // project add
        run_project_add("demo", Some(&dbs)).expect("project add");
        // agent add
        run_agent_add("demo", "backend", "backend", "gemini", "g-1.5", &vec!["Edit".into()], "sp", Some(&dbs)).expect("agent add");
    }
}
