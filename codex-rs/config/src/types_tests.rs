use super::*;
use pretty_assertions::assert_eq;

#[test]
fn deserialize_skill_config_with_name_selector() {
    let cfg: SkillConfig = toml::from_str(
        r#"
            name = "github:yeet"
            enabled = false
        "#,
    )
    .expect("should deserialize skill config with name selector");

    assert_eq!(cfg.name.as_deref(), Some("github:yeet"));
    assert_eq!(cfg.path, None);
    assert!(!cfg.enabled);
}

#[test]
fn deserialize_skill_config_with_path_selector() {
    let tempdir = tempfile::tempdir().expect("tempdir");
    let skill_path = tempdir.path().join("skills").join("demo").join("SKILL.md");
    let cfg: SkillConfig = toml::from_str(&format!(
        r#"
            path = {path:?}
            enabled = false
        "#,
        path = skill_path.display().to_string(),
    ))
    .expect("should deserialize skill config with path selector");

    assert_eq!(
        cfg,
        SkillConfig {
            path: Some(
                AbsolutePathBuf::from_absolute_path(&skill_path)
                    .expect("skill path should be absolute"),
            ),
            name: None,
            enabled: false,
        }
    );
}

#[test]
fn memories_config_clamps_count_limits_to_nonzero_values() {
    let config = MemoriesConfig::from(MemoriesToml {
        max_raw_memories_for_consolidation: Some(0),
        max_rollouts_per_startup: Some(0),
        ..Default::default()
    });

    assert_eq!(
        config,
        MemoriesConfig {
            max_raw_memories_for_consolidation: 1,
            max_rollouts_per_startup: 1,
            ..MemoriesConfig::default()
        }
    );
}

#[test]
fn sandbox_workspace_write_defaults_read_only_access_to_full_access() {
    use codex_protocol::protocol::ReadOnlyAccess;

    // A config that omits read_only_access must still deserialize and
    // round-trip to the historical FullAccess default so existing
    // config.toml files keep working after the field was added.
    let cfg: SandboxWorkspaceWrite = toml::from_str(
        r#"
            writable_roots = []
            network_access = false
        "#,
    )
    .expect("existing config shapes must still deserialize");

    assert_eq!(cfg.read_only_access, ReadOnlyAccess::FullAccess);
}

#[test]
fn sandbox_workspace_write_deserializes_restricted_read_only_access() {
    use codex_protocol::protocol::ReadOnlyAccess;

    let tmp = tempfile::tempdir().expect("tempdir");
    let extra_root = tmp.path().join("extra");
    std::fs::create_dir_all(&extra_root).expect("create tempdir subdir");
    let extra_root_str = extra_root.display().to_string();

    // Note the `type = "restricted"` TOML form, matching the serde
    // `#[serde(tag = "type", rename_all = "kebab-case")]` on ReadOnlyAccess.
    let cfg: SandboxWorkspaceWrite = toml::from_str(&format!(
        r#"
            writable_roots = []

            [read_only_access]
            type = "restricted"
            include_platform_defaults = true
            readable_roots = [{path:?}]
        "#,
        path = extra_root_str,
    ))
    .expect("should deserialize restricted read_only_access");

    match cfg.read_only_access {
        ReadOnlyAccess::Restricted {
            include_platform_defaults,
            readable_roots,
        } => {
            assert!(include_platform_defaults);
            assert_eq!(readable_roots.len(), 1);
            assert_eq!(
                readable_roots[0].as_path(),
                extra_root.canonicalize().unwrap_or(extra_root.clone())
            );
        }
        other => panic!(
            "expected Restricted read_only_access, got {other:?}"
        ),
    }
}
