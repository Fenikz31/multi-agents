//! Integration tests for configuration

use crate::utils::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_defaults_with_env_dir() {
        // Prepare temp config dir
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let cfg_dir = dir.join("config");
        std::fs::create_dir_all(&cfg_dir).unwrap();
        let project_p = cfg_dir.join("project.yaml");
        let providers_p = cfg_dir.join("providers.yaml");
        std::fs::write(&project_p, "schema_version: 1\nproject: demo\nagents: []\n").unwrap();
        std::fs::write(&providers_p, "schema_version: 1\nproviders: {}\n").unwrap();

        // Point resolution to this temp dir
        std::env::set_var("MULTI_AGENTS_CONFIG_DIR", cfg_dir.to_string_lossy().to_string());
        let (pr, pv) = resolve_config_paths(None, None).expect("resolve");
        assert_eq!(std::path::Path::new(&pr), project_p);
        assert_eq!(std::path::Path::new(&pv), providers_p);
        std::env::remove_var("MULTI_AGENTS_CONFIG_DIR");
    }

    #[test]
    fn test_config_autodetect_missing_config_error() {
        // Test missing config error handling
        let result = resolve_config_paths(Some("/nonexistent/project.yaml"), Some("/nonexistent/providers.yaml"));
        assert!(result.is_err(), "Should return error for missing config files");
        
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("project config not found"), "Should mention project config");
        assert!(error_msg.contains("--project-file"), "Should mention project file flag");
        assert!(error_msg.contains("MULTI_AGENTS_PROJECT_FILE"), "Should mention env var");
    }

    #[test]
    fn test_config_autodetect_file_extensions() {
        // Test that both .yaml and .yml extensions are supported
        let temp_dir = std::env::temp_dir().join("multi-agents-test-extensions");
        let _ = std::fs::create_dir_all(&temp_dir);
        
        // Create test config files with .yml extension
        let project_file = temp_dir.join("project.yml");
        let providers_file = temp_dir.join("providers.yml");
        std::fs::write(&project_file, "project: test").unwrap();
        std::fs::write(&providers_file, "providers: {}").unwrap();
        
        std::env::set_var("MULTI_AGENTS_CONFIG_DIR", temp_dir.to_string_lossy().to_string());
        
        let (proj_path, prov_path) = resolve_config_paths(None, None).unwrap();
        assert_eq!(proj_path, project_file.to_string_lossy().to_string());
        assert_eq!(prov_path, providers_file.to_string_lossy().to_string());
        
        // Cleanup
        std::env::remove_var("MULTI_AGENTS_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
