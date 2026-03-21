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
        extra_args: None,
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
        extra_args: None,
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
        extra_args: None,
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
        extra_args: None,
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
        extra_args: None,
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
        extra_args: None,
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
        extra_args: None,
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

#[test]
fn fmt_all_fields_set_no_leakage() {
    use crate::tools::CargoFmt;
    use std::collections::HashMap;

    let fmt = CargoFmt {
        check: Some(false),
        toolchain: Some("nightly".into()),
        cargo_env: Some(HashMap::from([("RUSTFLAGS".into(), "-Awarnings".into())])),
        extra_args: None,
    };
    let args = fmt.build_args();
    assert_eq!(
        args,
        vec!["fmt"],
        "build_args should only contain fmt args, not toolchain or env"
    );
    assert!(
        !args.contains(&"nightly".to_string()),
        "toolchain should not appear in build_args"
    );
}

#[test]
fn doc_default_produces_no_deps() {
    use crate::tools::CargoDoc;

    let doc = CargoDoc::default();
    let args = doc.build_args();
    assert_eq!(
        args,
        vec!["doc", "--no-deps"],
        "default CargoDoc should produce ['doc', '--no-deps']"
    );
}

#[test]
fn doc_with_deps_no_flag() {
    use crate::tools::CargoDoc;

    let doc = CargoDoc {
        no_deps: Some(false),
        ..CargoDoc::default()
    };
    let args = doc.build_args();
    assert_eq!(
        args,
        vec!["doc"],
        "no_deps: false should produce ['doc'] without --no-deps"
    );
}

#[test]
fn doc_private_items_flag() {
    use crate::tools::CargoDoc;

    let doc = CargoDoc {
        document_private_items: Some(true),
        ..CargoDoc::default()
    };
    let args = doc.build_args();
    assert!(
        args.contains(&"--document-private-items".to_string()),
        "document_private_items: true should include --document-private-items"
    );
}

#[test]
fn doc_with_package() {
    use crate::tools::CargoDoc;

    let doc = CargoDoc {
        package: Some("foo".into()),
        ..CargoDoc::default()
    };
    let args = doc.build_args();
    assert!(
        args.contains(&"--package".to_string()),
        "should contain --package flag"
    );
    assert!(
        args.contains(&"foo".to_string()),
        "should contain package name"
    );
}

#[test]
fn doc_empty_package_passed_through() {
    use crate::tools::CargoDoc;

    let doc = CargoDoc {
        package: Some("".into()),
        ..CargoDoc::default()
    };
    let args = doc.build_args();
    assert_eq!(
        args,
        vec!["doc", "--package", "", "--no-deps"],
        "empty package string should be passed through verbatim"
    );
}

#[test]
fn doc_package_resembling_flag() {
    use crate::tools::CargoDoc;

    let doc = CargoDoc {
        package: Some("--help".into()),
        ..CargoDoc::default()
    };
    let args = doc.build_args();
    assert_eq!(
        args,
        vec!["doc", "--package", "--help", "--no-deps"],
        "flag-like package name should be passed through verbatim"
    );
}

#[test]
fn doc_all_fields_set() {
    use crate::tools::CargoDoc;

    let doc = CargoDoc {
        package: Some("my-lib".into()),
        no_deps: Some(true),
        document_private_items: Some(true),
        toolchain: Some("nightly".into()),
        cargo_env: None,
        extra_args: None,
    };
    let args = doc.build_args();
    assert_eq!(
        args,
        vec![
            "doc",
            "--package",
            "my-lib",
            "--no-deps",
            "--document-private-items"
        ],
        "all fields set should produce correct arg ordering"
    );
    // toolchain is NOT in build_args (handled by execute via create_cargo_command)
    assert!(
        !args.contains(&"nightly".to_string()),
        "toolchain should not appear in build_args"
    );
}

// ── extra_args tests ──────────────────────────────────────────────────

#[test]
fn clippy_extra_args_before_separator() {
    use crate::tools::CargoClippy;

    let clippy = CargoClippy {
        package: None,
        toolchain: None,
        fix: None,
        all_targets: None,
        cargo_env: None,
        extra_args: Some(vec!["--no-default-features".into()]),
    };
    let args = clippy.build_args();
    let extra_pos = args
        .iter()
        .position(|a| a == "--no-default-features")
        .expect("--no-default-features should be present");
    let separator_pos = args
        .iter()
        .position(|a| a == "--")
        .expect("-- separator should be present");
    assert!(
        extra_pos < separator_pos,
        "extra_args must appear before -- separator, got extra at {extra_pos}, separator at {separator_pos}"
    );
}

#[test]
fn clippy_extra_args_empty_vec_unchanged() {
    use crate::tools::CargoClippy;

    let with_empty = CargoClippy {
        package: None,
        toolchain: None,
        fix: None,
        all_targets: None,
        cargo_env: None,
        extra_args: Some(vec![]),
    };
    let with_none = CargoClippy {
        package: None,
        toolchain: None,
        fix: None,
        all_targets: None,
        cargo_env: None,
        extra_args: None,
    };
    assert_eq!(
        with_empty.build_args(),
        with_none.build_args(),
        "extra_args: Some(vec![]) must produce same args as extra_args: None"
    );
}

#[test]
fn test_standard_extra_args_before_separator() {
    use crate::tools::CargoTest;

    let t = CargoTest {
        package: None,
        test_name: None,
        no_capture: Some(true),
        toolchain: None,
        cargo_env: None,
        use_nextest: None,
        extra_args: Some(vec!["--lib".into()]),
    };
    let args = t.build_args();
    let extra_pos = args
        .iter()
        .position(|a| a == "--lib")
        .expect("--lib should be present");
    let separator_pos = args
        .iter()
        .position(|a| a == "--")
        .expect("-- separator should be present with no_capture");
    assert!(
        extra_pos < separator_pos,
        "extra_args must appear before -- separator, got extra at {extra_pos}, separator at {separator_pos}"
    );
}

#[test]
fn test_standard_extra_args_no_separator() {
    use crate::tools::CargoTest;

    let t = CargoTest {
        package: None,
        test_name: None,
        no_capture: None,
        toolchain: None,
        cargo_env: None,
        use_nextest: None,
        extra_args: Some(vec!["--lib".into()]),
    };
    let args = t.build_args();
    assert!(
        args.contains(&"--lib".to_string()),
        "--lib should be present"
    );
    assert!(
        !args.contains(&"--".to_string()),
        "-- separator should not be present without no_capture"
    );
}

#[test]
fn test_nextest_extra_args() {
    use crate::tools::CargoTest;

    let t = CargoTest {
        package: None,
        test_name: None,
        no_capture: None,
        toolchain: None,
        cargo_env: None,
        use_nextest: Some(true),
        extra_args: Some(vec!["--lib".into()]),
    };
    let args = t.build_args();
    let run_pos = args
        .iter()
        .position(|a| a == "run")
        .expect("run should be present for nextest");
    let extra_pos = args
        .iter()
        .position(|a| a == "--lib")
        .expect("--lib should be present");
    assert!(
        extra_pos > run_pos,
        "--lib must appear after 'run', got run at {run_pos}, extra at {extra_pos}"
    );
}

#[test]
fn fmt_extra_args_appended() {
    use crate::tools::CargoFmt;

    let fmt = CargoFmt {
        check: None,
        toolchain: None,
        cargo_env: None,
        extra_args: Some(vec!["--config-path".into(), "custom.toml".into()]),
    };
    let args = fmt.build_args();
    // Default check=true, so --check should be present, then extra_args after
    assert_eq!(
        args,
        vec!["fmt", "--check", "--config-path", "custom.toml"],
        "extra_args should appear after tool flags"
    );
}

#[test]
fn doc_extra_args_appended() {
    use crate::tools::CargoDoc;

    let doc = CargoDoc {
        package: None,
        no_deps: None,
        document_private_items: None,
        toolchain: None,
        cargo_env: None,
        extra_args: Some(vec!["--all-features".into()]),
    };
    let args = doc.build_args();
    assert_eq!(
        args,
        vec!["doc", "--no-deps", "--all-features"],
        "extra_args should appear after tool flags"
    );
}

// ── adversarial extra_args tests ──────────────────────────────────────

#[test]
fn clippy_extra_args_containing_separator() {
    use crate::tools::CargoClippy;

    // Semantically hostile: user passes "--" as an extra_arg.
    // Per anti-pattern: no validation. It should pass through verbatim,
    // creating two "--" in the args. The first "--" (from extra_args) and
    // the second (from clippy's own separator) are both present.
    let clippy = CargoClippy {
        package: None,
        toolchain: None,
        fix: None,
        all_targets: None,
        cargo_env: None,
        extra_args: Some(vec!["--".into(), "-W".into(), "clippy::all".into()]),
    };
    let args = clippy.build_args();
    // extra_args should appear before clippy's own "--"
    // Expected: ["clippy", "--", "-W", "clippy::all", "--", "-D", "warnings"]
    assert_eq!(
        args,
        vec!["clippy", "--", "-W", "clippy::all", "--", "-D", "warnings"],
        "extra_args with '--' should pass through verbatim before clippy's own separator"
    );
}

#[test]
fn clippy_extra_args_multiple_before_separator() {
    use crate::tools::CargoClippy;

    // Redundant: multiple extra_args including duplicates
    let clippy = CargoClippy {
        package: Some("foo".into()),
        toolchain: None,
        fix: None,
        all_targets: Some(true),
        cargo_env: None,
        extra_args: Some(vec![
            "--no-default-features".into(),
            "--features".into(),
            "serde".into(),
            "--features".into(),
            "tokio".into(),
        ]),
    };
    let args = clippy.build_args();
    let separator_pos = args
        .iter()
        .position(|a| a == "--")
        .expect("-- separator should be present");
    // All extra_args must be before the separator
    for extra in &["--no-default-features", "--features", "serde", "tokio"] {
        let pos = args
            .iter()
            .position(|a| a == extra)
            .unwrap_or_else(|| panic!("{extra} should be present"));
        assert!(
            pos < separator_pos,
            "{extra} at {pos} must be before -- at {separator_pos}"
        );
    }
    // Duplicates preserved (--features appears twice)
    assert_eq!(
        args.iter().filter(|a| *a == "--features").count(),
        2,
        "duplicate --features should be preserved, not deduped"
    );
}

#[test]
fn test_extra_args_with_empty_string() {
    use crate::tools::CargoTest;

    // Encoding boundary: empty string in extra_args
    let t = CargoTest {
        package: None,
        test_name: None,
        no_capture: None,
        toolchain: None,
        cargo_env: None,
        use_nextest: None,
        extra_args: Some(vec!["".into()]),
    };
    let args = t.build_args();
    assert_eq!(
        args,
        vec!["test", ""],
        "empty string extra_arg should be passed through verbatim"
    );
}
