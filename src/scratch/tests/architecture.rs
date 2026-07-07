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

#[test]
fn responder_chain_uses_service_responders_not_framework_fallbacks() {
    let scratch_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scratch");
    let forbidden = [
        format!("{}{}", "trait ", "Framework"),
        format!("{}{}", "with_", "framework"),
        format!("{}{}", "responder::", "Framework"),
        format!("{}{}", "mod ", "framework;"),
        format!("{}{}", "services/", "framework.rs"),
        format!("{}{}", "framework_", "command"),
        format!("{}{}", "Framework", "Runtime"),
        format!("{}{}", "framework_", "view"),
        format!("{}{}", "framework_", "shell"),
        format!("{}{}", "framework_", "icon"),
    ];

    assert_source_patterns_absent(&scratch_dir, &forbidden);
}

#[test]
fn runtime_services_use_typed_provider_targets() {
    let services_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scratch")
        .join("runtime")
        .join("services");
    let forbidden = [
        "AnyTarget::new::<".to_owned(),
        "target::args::<".to_owned(),
        "target::args_box::<".to_owned(),
        "text_target!(".to_owned(),
        "fn command_name(".to_owned(),
        format!("{}{}", "framework_", "command"),
    ];

    assert_source_patterns_absent(&services_dir, &forbidden);
}

#[test]
fn focused_text_service_stays_behind_runtime_boundary() {
    let runtime_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scratch")
        .join("runtime");
    let services_mod = runtime_dir.join("services").join("mod.rs");
    let input_mod = runtime_dir.join("input").join("mod.rs");
    let services_source =
        std::fs::read_to_string(&services_mod).expect("runtime services module should read");
    let input_source =
        std::fs::read_to_string(&input_mod).expect("runtime input module should read");

    assert!(
        !services_source.contains("pub(in crate::scratch::runtime) mod text;"),
        "{} must keep focused text service private to runtime services",
        services_mod.display()
    );
    assert!(
        !input_source.contains("pub(in crate::scratch::runtime) mod text;"),
        "{} must keep text input internals private to runtime input",
        input_mod.display()
    );
    assert_source_patterns_absent(&runtime_dir, &[format!("{}{}", "services::", "text")]);
}

#[test]
fn composition_tree_owns_identity_not_behavior() {
    let scratch_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scratch");
    let composition_dir = scratch_dir.join("composition");
    let widget_dir = scratch_dir.join("widget");

    assert_source_patterns_absent(
        &composition_dir,
        &[
            "command::Registry".to_owned(),
            "runtime::".to_owned(),
            "platform::".to_owned(),
            "fn mount".to_owned(),
            "fn unmount".to_owned(),
        ],
    );
    assert_source_patterns_absent(
        &widget_dir,
        &[
            "composition::NodeId".to_owned(),
            "crate::scratch::composition".to_owned(),
            "fn mount".to_owned(),
            "fn unmount".to_owned(),
        ],
    );
}

#[test]
fn retained_node_identity_replaces_structural_command_fallbacks() {
    let scratch_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("scratch");

    assert_source_patterns_absent(
        &scratch_dir,
        &[
            format!("{}{}", "Command", "Path"),
            format!("{}{}", "command", "_path"),
            format!("{}{}", "path_", "pointer_target"),
        ],
    );
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

fn assert_source_patterns_absent(path: &std::path::Path, patterns: &[String]) {
    for entry in std::fs::read_dir(path).expect("scratch source directory should be readable") {
        let path = entry
            .expect("scratch source entry should be readable")
            .path();
        if path.is_dir() {
            assert_source_patterns_absent(&path, patterns);
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("scratch source file should read");
        for pattern in patterns {
            assert!(
                !source.contains(pattern),
                "{} must not contain stale routing concept {pattern}",
                path.display()
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
