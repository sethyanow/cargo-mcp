use crate::tools::Tools;
use mcplease::traits::AsToolsList;

#[test]
fn tools_doesnt_panic() {
    Tools::tools_list();
}

#[test]
fn clippy_all_targets_produces_flag() {
    use crate::tools::CargoClippy;

    let clippy = CargoClippy {
        package: None,
        toolchain: None,
        fix: None,
        cargo_env: None,
        all_targets: Some(true),
    };
    let args = clippy.build_args();
    let all_targets_pos = args
        .iter()
        .position(|a| a == "--all-targets")
        .expect("--all-targets should be present in args");
    let separator_pos = args
        .iter()
        .position(|a| a == "--")
        .expect("-- separator should be present in args");
    assert!(
        all_targets_pos < separator_pos,
        "--all-targets must appear before -- separator"
    );
}

#[test]
fn clippy_default_has_no_all_targets() {
    use crate::tools::CargoClippy;

    let clippy = CargoClippy {
        package: None,
        toolchain: None,
        fix: None,
        cargo_env: None,
        all_targets: None,
    };
    let args = clippy.build_args();
    assert!(
        !args.contains(&"--all-targets".to_string()),
        "--all-targets should not be present when None"
    );
}

#[test]
fn clippy_all_fields_none_produces_minimal_args() {
    use crate::tools::CargoClippy;

    let clippy = CargoClippy {
        package: None,
        toolchain: None,
        fix: None,
        cargo_env: None,
        all_targets: None,
    };
    let args = clippy.build_args();
    assert_eq!(args, vec!["clippy", "--", "-D", "warnings"]);
}

#[test]
fn clippy_all_fields_set_ordering() {
    use crate::tools::CargoClippy;

    let clippy = CargoClippy {
        package: Some("my-crate".into()),
        toolchain: Some("nightly".into()),
        fix: Some(true),
        cargo_env: None,
        all_targets: Some(true),
    };
    let args = clippy.build_args();
    let separator_pos = args
        .iter()
        .position(|a| a == "--")
        .expect("-- separator should be present");
    // All cargo flags must appear before --
    assert!(args[..separator_pos].contains(&"--package".to_string()));
    assert!(args[..separator_pos].contains(&"my-crate".to_string()));
    assert!(args[..separator_pos].contains(&"--fix".to_string()));
    assert!(args[..separator_pos].contains(&"--all-targets".to_string()));
    // Clippy lint args after --
    assert!(args[separator_pos..].contains(&"-D".to_string()));
    assert!(args[separator_pos..].contains(&"warnings".to_string()));
    // toolchain is NOT in build_args (handled by execute via create_cargo_command)
    assert!(!args.contains(&"nightly".to_string()));
}

#[test]
fn clippy_explicit_false_no_all_targets() {
    use crate::tools::CargoClippy;

    let clippy = CargoClippy {
        package: None,
        toolchain: None,
        fix: None,
        cargo_env: None,
        all_targets: Some(false),
    };
    let args = clippy.build_args();
    assert!(
        !args.contains(&"--all-targets".to_string()),
        "--all-targets should not be present when explicitly false"
    );
}

#[test]
fn working_directory_is_per_process() {
    use crate::state::CargoTools;
    use std::path::PathBuf;

    let mut instance_a = CargoTools::new().unwrap();
    instance_a
        .set_working_directory(PathBuf::from("/tmp/project-a"), None)
        .unwrap();

    // A second instance must NOT see instance A's working directory
    let mut instance_b = CargoTools::new().unwrap();
    assert_eq!(
        instance_b.get_context(None).unwrap(),
        None,
        "working directory should not bleed between instances"
    );

    // Verify A still has its own value
    assert_eq!(
        instance_a.get_context(None).unwrap(),
        Some(PathBuf::from("/tmp/project-a")),
    );
}
