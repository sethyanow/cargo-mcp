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

#[test]
fn test_nextest_mode_produces_nextest_run() {
    use crate::tools::CargoTest;

    let test_cmd = CargoTest {
        use_nextest: Some(true),
        ..CargoTest::default()
    };
    let args = test_cmd.build_args();
    assert_eq!(
        &args[..2],
        &["nextest", "run"],
        "nextest mode should produce args starting with ['nextest', 'run']"
    );
    assert!(
        !args.contains(&"test".to_string()),
        "nextest mode should not contain 'test' subcommand"
    );
}

#[test]
fn test_nextest_no_capture_flag() {
    use crate::tools::CargoTest;

    let test_cmd = CargoTest {
        use_nextest: Some(true),
        no_capture: Some(true),
        ..CargoTest::default()
    };
    let args = test_cmd.build_args();
    assert!(
        args.contains(&"--no-capture".to_string()),
        "nextest mode should use --no-capture flag"
    );
    assert!(
        !args.contains(&"--nocapture".to_string()),
        "nextest mode should not use --nocapture"
    );
    assert!(
        !args.contains(&"--".to_string()),
        "nextest mode should not have -- separator for no-capture"
    );
}

#[test]
fn test_standard_mode_no_capture() {
    use crate::tools::CargoTest;

    let test_cmd = CargoTest {
        no_capture: Some(true),
        ..CargoTest::default()
    };
    let args = test_cmd.build_args();
    assert_eq!(args[0], "test", "standard mode should start with 'test'");
    let separator_pos = args
        .iter()
        .position(|a| a == "--")
        .expect("standard mode should have -- separator");
    assert_eq!(
        args[separator_pos + 1],
        "--nocapture",
        "standard mode should use --nocapture after --"
    );
}

#[test]
fn test_explicit_false_no_nextest() {
    use crate::tools::CargoTest;

    let test_cmd = CargoTest {
        use_nextest: Some(false),
        ..CargoTest::default()
    };
    let args = test_cmd.build_args();
    assert_eq!(
        args[0], "test",
        "explicit false should produce standard 'test' subcommand"
    );
    assert!(
        !args.contains(&"nextest".to_string()),
        "explicit false should not contain 'nextest'"
    );
}

#[test]
fn test_nextest_all_fields() {
    use crate::tools::CargoTest;

    let test_cmd = CargoTest {
        use_nextest: Some(true),
        package: Some("foo".into()),
        test_name: Some("bar".into()),
        no_capture: Some(true),
        toolchain: Some("nightly".into()),
        cargo_env: None,
    };
    let args = test_cmd.build_args();
    assert_eq!(
        args,
        vec!["nextest", "run", "--package", "foo", "bar", "--no-capture"],
        "nextest with all fields should produce correct arg ordering"
    );
    // toolchain is NOT in build_args (handled by execute via create_cargo_command)
    assert!(
        !args.contains(&"nightly".to_string()),
        "toolchain should not appear in build_args"
    );
}

#[test]
fn test_all_fields_none_produces_minimal_test_args() {
    use crate::tools::CargoTest;

    let test_cmd = CargoTest::default();
    let args = test_cmd.build_args();
    assert_eq!(
        args,
        vec!["test"],
        "default CargoTest should produce only ['test']"
    );
}

#[test]
fn test_standard_all_fields_set_ordering() {
    use crate::tools::CargoTest;

    let test_cmd = CargoTest {
        use_nextest: None,
        package: Some("my-crate".into()),
        test_name: Some("test_foo".into()),
        no_capture: Some(true),
        toolchain: Some("stable".into()),
        cargo_env: None,
    };
    let args = test_cmd.build_args();
    assert_eq!(
        args,
        vec![
            "test",
            "--package",
            "my-crate",
            "test_foo",
            "--",
            "--nocapture"
        ],
        "standard mode with all fields should produce correct ordering"
    );
}

#[test]
fn test_package_with_special_chars() {
    use crate::tools::CargoTest;

    let test_cmd = CargoTest {
        package: Some("my crate".into()),
        ..CargoTest::default()
    };
    let args = test_cmd.build_args();
    assert_eq!(
        args,
        vec!["test", "--package", "my crate"],
        "package with spaces should be passed through verbatim"
    );
}

#[test]
fn test_name_resembling_flag() {
    use crate::tools::CargoTest;

    let test_cmd = CargoTest {
        test_name: Some("--help".into()),
        ..CargoTest::default()
    };
    let args = test_cmd.build_args();
    assert_eq!(
        args,
        vec!["test", "--help"],
        "test_name that looks like a flag should be passed through verbatim"
    );
}

#[test]
fn fmt_check_mode_produces_check_flag() {
    use crate::tools::CargoFmt;

    let fmt = CargoFmt {
        check: None,
        ..CargoFmt::default()
    };
    let args = fmt.build_args();
    assert_eq!(
        args,
        vec!["fmt", "--check"],
        "default (None) check should produce ['fmt', '--check']"
    );
}

#[test]
fn fmt_write_mode_no_check_flag() {
    use crate::tools::CargoFmt;

    let fmt = CargoFmt {
        check: Some(false),
        ..CargoFmt::default()
    };
    let args = fmt.build_args();
    assert_eq!(
        args,
        vec!["fmt"],
        "check: false should produce ['fmt'] without --check"
    );
}

#[test]
fn fmt_explicit_true_check_flag() {
    use crate::tools::CargoFmt;

    let fmt = CargoFmt {
        check: Some(true),
        ..CargoFmt::default()
    };
    let args = fmt.build_args();
    assert_eq!(
        args,
        vec!["fmt", "--check"],
        "check: Some(true) should produce ['fmt', '--check']"
    );
}

#[test]
fn fmt_default_produces_minimal_args() {
    use crate::tools::CargoFmt;

    let fmt = CargoFmt::default();
    let args = fmt.build_args();
    assert_eq!(
        args,
        vec!["fmt", "--check"],
        "CargoFmt::default() should produce ['fmt', '--check']"
    );
}
