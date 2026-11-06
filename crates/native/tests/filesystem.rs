use std::{
    fs::{self, File},
    io::{Cursor, Seek, SeekFrom},
    path::Path,
};

use installwizard_core::{Cancellation, append_package, inspect_package};
use installwizard_native::{
    ExportOptions, Overwrite, Project, Source, build, collect, extract, host_target,
};

fn project(source: &Path) -> Project {
    Project {
        name: "Test".into(),
        sources: vec![Source {
            source: source.into(),
            destination: "project".into(),
        }],
        ..Default::default()
    }
}
fn package(project: &Project) -> Cursor<Vec<u8>> {
    let mut archive = Cursor::new(Vec::new());
    let manifest = collect(&project.sources)
        .unwrap()
        .write(&mut archive, project, &Cancellation::default(), |_| {})
        .unwrap();
    let mut output = Cursor::new(Vec::new());
    append_package(&mut output, archive.get_ref().as_slice(), &manifest).unwrap();
    output
}

#[test]
fn tree_roundtrip_includes_empty_hidden_unicode_and_executable() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::create_dir_all(source.join("empty/nested")).unwrap();
    fs::write(source.join(".hidden"), b"secret").unwrap();
    fs::write(source.join("日本語.txt"), b"hello world").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(source.join(".hidden"), fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut binary = package(&project(&source));
    let package = inspect_package(&mut binary).unwrap().unwrap();
    let destination = root.path().join("out");
    extract(
        &mut binary,
        &package,
        &destination,
        Overwrite::Refuse,
        &Cancellation::default(),
        |_| {},
    )
    .unwrap();
    assert!(destination.join("project/empty/nested").is_dir());
    assert_eq!(
        fs::read(destination.join("project/日本語.txt")).unwrap(),
        b"hello world"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(destination.join("project/.hidden"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
    }
}

#[test]
fn conflicts_preserve_existing_files_and_explicit_overwrite_works() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("a"), b"new").unwrap();
    let mut binary = package(&project(&source));
    let package = inspect_package(&mut binary).unwrap().unwrap();
    let destination = root.path().join("out");
    fs::create_dir_all(destination.join("project")).unwrap();
    fs::write(destination.join("project/a"), b"old").unwrap();
    assert!(
        extract(
            &mut binary,
            &package,
            &destination,
            Overwrite::Refuse,
            &Cancellation::default(),
            |_| {}
        )
        .is_err()
    );
    assert_eq!(fs::read(destination.join("project/a")).unwrap(), b"old");
    extract(
        &mut binary,
        &package,
        &destination,
        Overwrite::Replace,
        &Cancellation::default(),
        |_| {},
    )
    .unwrap();
    assert_eq!(fs::read(destination.join("project/a")).unwrap(), b"new");
}

#[cfg(unix)]
#[test]
fn destination_symlinks_cannot_escape_root() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("a"), b"new").unwrap();
    let mut binary = package(&project(&source));
    let package = inspect_package(&mut binary).unwrap().unwrap();
    let destination = root.path().join("out");
    let outside = root.path().join("outside");
    fs::create_dir(&destination).unwrap();
    fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, destination.join("project")).unwrap();
    assert!(
        extract(
            &mut binary,
            &package,
            &destination,
            Overwrite::Replace,
            &Cancellation::default(),
            |_| {}
        )
        .is_err()
    );
    assert!(!outside.join("a").exists());
    let linked = root.path().join("linked-source");
    std::os::unix::fs::symlink(&source, &linked).unwrap();
    assert!(collect(&project(&linked).sources).is_err());
}

#[test]
fn saved_project_resolves_paths_relative_to_its_file() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::create_dir(&source).unwrap();
    let path = root.path().join("project.json");
    project(&source).save(&path).unwrap();
    let loaded = Project::load(&path).unwrap();
    assert_eq!(loaded.sources[0].source, source);
}

#[test]
fn nested_output_is_rejected() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::create_dir(&source).unwrap();
    let options = ExportOptions {
        output: source.join("output"),
        templates: None,
        overwrite: false,
    };
    assert!(
        build(
            &project(&source),
            &options,
            &Cancellation::default(),
            |_| {}
        )
        .is_err()
    );
}

#[test]
fn corrupted_archive_is_rejected_before_destination_is_created() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("a"), b"new").unwrap();
    let mut binary = package(&project(&source));
    let mut package = inspect_package(&mut binary).unwrap().unwrap();
    package
        .manifest
        .entries
        .iter_mut()
        .find(|e| !e.directory)
        .unwrap()
        .sha256 = "0".repeat(64);
    let destination = root.path().join("out");
    assert!(
        extract(
            &mut binary,
            &package,
            &destination,
            Overwrite::Refuse,
            &Cancellation::default(),
            |_| {}
        )
        .is_err()
    );
    assert!(!destination.exists());
}

#[test]
fn native_export_uses_one_archive_for_multiple_targets() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("a"), b"hello").unwrap();
    let templates = root.path().join("templates");
    fs::create_dir(&templates).unwrap();
    let mut header = vec![0; 64];
    header[..6].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1]);
    header[18] = 62;
    fs::write(templates.join("wizard-linux-x64"), &header).unwrap();
    header[18] = 183;
    fs::write(templates.join("wizard-linux-arm64"), &header).unwrap();
    let mut project = project(&source);
    project.targets = vec![
        installwizard_core::Target::LinuxX64,
        installwizard_core::Target::LinuxArm64,
    ];
    let options = ExportOptions {
        output: root.path().join("output"),
        templates: Some(templates),
        overwrite: false,
    };
    let paths = build(&project, &options, &Cancellation::default(), |_| {}).unwrap();
    assert_eq!(paths.len(), 2);
    for path in paths {
        let mut input = File::open(path).unwrap();
        let package = inspect_package(&mut input).unwrap().unwrap();
        input.seek(SeekFrom::Start(0)).unwrap();
        package
            .verify(&mut input, &Cancellation::default(), |_| {})
            .unwrap();
    }
    let _ = host_target();
}
