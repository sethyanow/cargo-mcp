use crate::tools::Tools;
use mcplease::traits::AsToolsList;

#[test]
fn tools_doesnt_panic() {
    Tools::tools_list();
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
