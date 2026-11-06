use std::{
    io::{Cursor, Seek, SeekFrom, Write},
    sync::Arc,
};

use installwizard_core::{
    Cancellation, InputEntry, Manifest, Target, append_package, create_archive, inspect_package,
    memory_artifact, validate_path,
};

fn sample() -> (Cursor<Vec<u8>>, Manifest) {
    let entries = vec![
        InputEntry {
            path: "assets".into(),
            directory: true,
            size: 0,
            executable: false,
        },
        InputEntry {
            path: "assets/hello.txt".into(),
            directory: false,
            size: 5,
            executable: true,
        },
    ];
    let bytes: Arc<[u8]> = Arc::from(b"hello".as_slice());
    let mut zip = Cursor::new(Vec::new());
    let manifest = create_archive(
        &mut zip,
        "Example",
        None,
        &entries,
        |_| Ok(Box::new(Cursor::new(bytes.clone()))),
        &Cancellation::default(),
        |_| {},
    )
    .unwrap();
    (zip, manifest)
}

#[test]
fn embedded_roundtrip_and_integrity() {
    let (zip, manifest) = sample();
    let mut binary = Cursor::new(b"executable-prefix".to_vec());
    append_package(&mut binary, zip.get_ref().as_slice(), &manifest).unwrap();
    let package = inspect_package(&mut binary).unwrap().unwrap();
    assert_eq!(package.archive_offset, 17);
    assert_eq!(package.manifest.entries[1].path, "assets/hello.txt");
    package
        .verify(&mut binary, &Cancellation::default(), |_| {})
        .unwrap();
    let mut copied = Vec::new();
    package.copy_archive(&mut binary, &mut copied).unwrap();
    assert_eq!(copied, *zip.get_ref());
}

#[test]
fn forged_manifest_digest_is_rejected() {
    let (zip, mut manifest) = sample();
    manifest.entries[1].sha256 = "0".repeat(64);
    let mut binary = Cursor::new(Vec::new());
    append_package(&mut binary, zip.get_ref().as_slice(), &manifest).unwrap();
    let package = inspect_package(&mut binary).unwrap().unwrap();
    assert!(
        package
            .verify(&mut binary, &Cancellation::default(), |_| {})
            .is_err()
    );
}

#[test]
fn footer_overflows_and_truncation_are_rejected() {
    let mut input = Cursor::new(Vec::new());
    input.write_all(&u64::MAX.to_le_bytes()).unwrap();
    input.write_all(&1_u64.to_le_bytes()).unwrap();
    input.write_all(&1_u64.to_le_bytes()).unwrap();
    input.write_all(b"IWPKG001").unwrap();
    assert!(inspect_package(&mut input).is_err());
    input.seek(SeekFrom::Start(0)).unwrap();
    assert!(
        inspect_package(&mut Cursor::new(b"short"))
            .unwrap()
            .is_none()
    );
}

#[test]
fn portable_names_reject_traversal_and_windows_devices() {
    for path in [
        "../file", "/root", "C:/file", "a\\b", "a/./b", "a//b", "CON.txt", "a/NUL", "com1.log",
        "x.", "x ", "x\n",
    ] {
        assert!(validate_path(path).is_err(), "Accepted {path:?}");
    }
    for path in [
        "assets/日本語.txt",
        "folder/.hidden",
        "file..txt",
        "company/data",
    ] {
        validate_path(path).unwrap();
    }
}

#[test]
fn collisions_include_case_unicode_and_parent_files() {
    let (_, base) = sample();
    for (a, b) in [("A", "a"), ("café", "cafe\u{301}"), ("file", "file/child")] {
        let mut manifest = base.clone();
        let mut file = manifest.entries[1].clone();
        file.path = a.into();
        let mut other = file.clone();
        other.path = b.into();
        manifest.entries = vec![file, other];
        assert!(manifest.validate().is_err(), "Accepted {a:?} and {b:?}");
    }
}

#[test]
fn cancellation_and_changing_input_fail_before_finish() {
    let entries = [InputEntry {
        path: "x".into(),
        directory: false,
        size: 3,
        executable: false,
    }];
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(
        create_archive(
            Cursor::new(Vec::new()),
            "x",
            None,
            &entries,
            |_| Ok(Box::new(Cursor::new(b"abc".to_vec()))),
            &cancel,
            |_| {}
        )
        .is_err()
    );
    assert!(
        create_archive(
            Cursor::new(Vec::new()),
            "x",
            None,
            &entries,
            |_| Ok(Box::new(Cursor::new(b"abcd".to_vec()))),
            &Cancellation::default(),
            |_| {}
        )
        .is_err()
    );
}

#[test]
fn macos_bundle_has_executable_resource_and_escaped_name() {
    let (zip, mut manifest) = sample();
    manifest.name = "A < B & C".into();
    let mut template = vec![0_u8; 64];
    template[..8].copy_from_slice(&[0xcf, 0xfa, 0xed, 0xfe, 12, 0, 0, 1]);
    let output = memory_artifact(&template, zip.get_ref(), &manifest, Target::MacosArm64).unwrap();
    let mut bundle = zip::ZipArchive::new(Cursor::new(output)).unwrap();
    let file = bundle
        .by_name("InstallWizard.app/Contents/MacOS/InstallWizard")
        .unwrap();
    assert_eq!(file.unix_mode().unwrap() & 0o777, 0o755);
    drop(file);
    let mut resource = Cursor::new(Vec::new());
    std::io::copy(
        &mut bundle
            .by_name("InstallWizard.app/Contents/Resources/payload.iwpkg")
            .unwrap(),
        &mut resource,
    )
    .unwrap();
    let package = inspect_package(&mut resource).unwrap().unwrap();
    package
        .verify(&mut resource, &Cancellation::default(), |_| {})
        .unwrap();
}

#[test]
fn templates_must_match_architecture() {
    let (zip, manifest) = sample();
    assert!(memory_artifact(b"wrong", zip.get_ref(), &manifest, Target::WindowsX64).is_err());
}

#[test]
fn windows_icon_is_embedded_before_payload_without_corrupting_executable() {
    let (zip, mut manifest) = sample();
    let image = image::RgbaImage::from_pixel(16, 16, image::Rgba([19, 107, 98, 255]));
    let mut png = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    let icon = installwizard_core::normalize_icon(png.get_ref()).unwrap();
    manifest.icon_png = Some(icon.clone());
    let template = include_bytes!("fixtures/windows-x64.exe");
    let bytes = memory_artifact(template, zip.get_ref(), &manifest, Target::WindowsX64).unwrap();
    let image = editpe::Image::parse(bytes.as_slice()).unwrap();
    assert_eq!(
        image
            .resource_directory()
            .unwrap()
            .get_main_icon()
            .unwrap()
            .unwrap(),
        icon
    );
    let mut executable = Cursor::new(bytes);
    let package = inspect_package(&mut executable).unwrap().unwrap();
    package
        .verify(&mut executable, &Cancellation::default(), |_| {})
        .unwrap();
}
