#[test]
fn scratch_sources_do_not_import_legacy_framework_modules() {
    let scratch_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scratch");
    let legacy_modules = [
        "app",
        "ui",
        "widget",
        "command",
        "window",
        "native",
        "theme",
        "text_system",
    ];

    assert_no_legacy_framework_imports(&scratch_dir, &legacy_modules);
}

#[test]
fn renderer_dependencies_stay_in_native_platform() {
    let scratch_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scratch");
    let native_dir = scratch_dir.join("platform").join("native");
    let renderer_modules = ["geometry", "paint", "render"];

    assert_imports_only_under(&scratch_dir, &native_dir, &renderer_modules);
}

fn assert_no_legacy_framework_imports(path: &std::path::Path, modules: &[&str]) {
    for entry in std::fs::read_dir(path).expect("scratch source directory should be readable") {
        let path = entry
            .expect("scratch source entry should be readable")
            .path();
        if path.is_dir() {
            assert_no_legacy_framework_imports(&path, modules);
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("scratch source file should read");
        for module in modules {
            assert!(
                !source_imports_crate_module(&source, module),
                "{} must not import or reference legacy framework module {}",
                path.display(),
                module
            );
        }
    }
}

fn assert_imports_only_under(
    path: &std::path::Path,
    allowed_root: &std::path::Path,
    modules: &[&str],
) {
    for entry in std::fs::read_dir(path).expect("scratch source directory should be readable") {
        let path = entry
            .expect("scratch source entry should be readable")
            .path();
        if path.is_dir() {
            assert_imports_only_under(&path, allowed_root, modules);
            continue;
        }

        if path.starts_with(allowed_root)
            || path.extension().and_then(|extension| extension.to_str()) != Some("rs")
        {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("scratch source file should read");
        for module in modules {
            assert!(
                !source_imports_crate_module(&source, module),
                "{} must import crate::{} only under {}",
                path.display(),
                module,
                allowed_root.display()
            );
        }
    }
}

fn source_imports_crate_module(source: &str, module: &str) -> bool {
    if source.contains(&format!("crate::{module}")) {
        return true;
    }

    source.lines().any(|line| {
        let line = line.trim();
        if line.starts_with(&format!("use crate::{module}")) {
            return true;
        }

        let Some(grouped) = line
            .strip_prefix("use crate::{")
            .and_then(|line| line.strip_suffix(';'))
        else {
            return false;
        };

        grouped.split(',').any(|segment| {
            let segment = segment.trim();
            let root = segment
                .split_once("::")
                .map_or(segment, |(root, _)| root.trim())
                .split_once(" as ")
                .map_or_else(|| segment, |(root, _)| root.trim());
            root == module
        })
    })
}
