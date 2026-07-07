#[test]
fn promoted_framework_has_no_scratch_or_legacy_root_surface() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let retired = [
        "scratch",
        "app",
        "ui",
        "native",
        "action.rs",
        "event.rs",
        "path.rs",
        "pointer.rs",
    ];

    for entry in retired {
        let path = src_dir.join(entry);
        assert!(
            !path.exists(),
            "{} should be absent after promoting scratch to the root framework",
            path.display()
        );
    }
}

#[test]
fn root_source_tree_has_no_empty_concept_buckets() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    for entry in std::fs::read_dir(&src_dir).expect("source directory should be readable") {
        let path = entry.expect("source entry should be readable").path();
        if !path.is_dir() {
            continue;
        }

        let mut entries = std::fs::read_dir(&path).unwrap_or_else(|error| {
            panic!("{} should be readable: {error}", path.display());
        });
        assert!(
            entries.next().is_some(),
            "{} is an empty root-level concept bucket",
            path.display()
        );
    }
}

#[test]
fn renderer_dependencies_stay_in_native_platform() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let allowed_roots = [
        src_dir.join("platform").join("native"),
        src_dir.join("render"),
        src_dir.join("paint"),
        src_dir.join("paint_geometry"),
    ];
    let renderer_modules = ["paint", "render"];

    assert_imports_only_under_any(&src_dir, &allowed_roots, &renderer_modules);
}

#[test]
fn renderer_paint_vocabulary_stays_private() {
    let lib = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("lib.rs"),
    )
    .expect("crate root should read");

    assert!(
        !lib.contains("pub mod paint;"),
        "renderer-facing paint vocabulary should not be public framework API"
    );
    assert!(
        !lib.contains("pub mod paint_geometry;"),
        "renderer-space geometry should not be public framework API"
    );
}

#[test]
fn responder_chain_uses_service_responders_not_framework_fallbacks() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
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

    assert_source_patterns_absent(&src_dir, &forbidden);
}

#[test]
fn runtime_services_use_typed_provider_targets() {
    let services_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
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
        .join("runtime");
    let services_mod = runtime_dir.join("services").join("mod.rs");
    let input_mod = runtime_dir.join("input").join("mod.rs");
    let services_source =
        std::fs::read_to_string(&services_mod).expect("runtime services module should read");
    let input_source =
        std::fs::read_to_string(&input_mod).expect("runtime input module should read");

    assert!(
        !services_source.contains("pub(in crate::runtime) mod text;"),
        "{} must keep focused text service private to runtime services",
        services_mod.display()
    );
    assert!(
        !input_source.contains("pub(in crate::runtime) mod text;"),
        "{} must keep text input internals private to runtime input",
        input_mod.display()
    );
    assert_source_patterns_absent(&runtime_dir, &[format!("{}{}", "services::", "text")]);
}

#[test]
fn composition_tree_owns_identity_not_behavior() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let composition_dir = src_dir.join("composition");
    let widget_dir = src_dir.join("widget");

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
            "crate::composition".to_owned(),
            "fn mount".to_owned(),
            "fn unmount".to_owned(),
        ],
    );
}

#[test]
fn retained_node_identity_replaces_structural_command_fallbacks() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    assert_source_patterns_absent(
        &src_dir,
        &[
            format!("{}{}", "Command", "Path"),
            format!("{}{}", "command", "_path"),
            format!("{}{}", "path_", "pointer_target"),
            format!("{}{}", "pointer_target_", "at_path"),
            format!("{}{}", "without_", "retained_id"),
        ],
    );
}

#[test]
fn focus_traversal_goes_through_retained_composition() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let view_mod = std::fs::read_to_string(src_dir.join("view").join("mod.rs"))
        .expect("view module should read");
    let traversal = std::fs::read_to_string(src_dir.join("view").join("node").join("traversal.rs"))
        .expect("view node traversal module should read");

    for pattern in [
        "pub fn contains_enabled_focus(",
        "pub fn focus_order(",
        "pub fn next_focus(",
    ] {
        assert!(
            !view_mod.contains(pattern),
            "public view focus traversal must not bypass retained composition: {pattern}"
        );
    }

    for pattern in [
        "fn collect_focus_order(&self",
        "fn collect_floating_panel_focus_order(&self",
    ] {
        assert!(
            !traversal.contains(pattern),
            "view node traversal must not keep structural focus-order fallback: {pattern}"
        );
    }
}

#[test]
fn public_target_contract_uses_public_command_values() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let spec = std::fs::read_to_string(src_dir.join("command").join("spec.rs"))
        .expect("command spec module should read");
    let state = std::fs::read_to_string(src_dir.join("command").join("state.rs"))
        .expect("command state module should read");
    let response = std::fs::read_to_string(src_dir.join("response").join("mod.rs"))
        .expect("response module should read");

    for pattern in [
        "pub fn new(display_name: &'static str)",
        "pub fn shortcut(mut self",
        "pub fn key_chord(mut self",
    ] {
        assert!(
            spec.contains(pattern),
            "command registration Spec must remain constructible by app code: {pattern}"
        );
    }

    for pattern in [
        "pub fn enabled()",
        "pub fn disabled()",
        "pub fn hidden()",
        "pub fn checked(mut self",
    ] {
        assert!(
            state.contains(pattern),
            "command State must remain constructible by external Target implementations: {pattern}"
        );
    }

    for pattern in [
        "pub fn output(output: O)",
        "pub fn changed(output: O)",
        "pub fn failed(error: Error)",
        "pub fn into_result(self)",
    ] {
        assert!(
            response.contains(pattern),
            "Response must remain constructible/readable by external Target implementations: {pattern}"
        );
    }
}

fn assert_source_patterns_absent(path: &std::path::Path, patterns: &[String]) {
    for entry in std::fs::read_dir(path).expect("framework source directory should be readable") {
        let path = entry
            .expect("framework source entry should be readable")
            .path();
        if path.is_dir() {
            assert_source_patterns_absent(&path, patterns);
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("framework source file should read");
        for pattern in patterns {
            assert!(
                !source.contains(pattern),
                "{} must not contain stale routing concept {pattern}",
                path.display()
            );
        }
    }
}

fn assert_imports_only_under_any(
    path: &std::path::Path,
    allowed_roots: &[std::path::PathBuf],
    modules: &[&str],
) {
    for entry in std::fs::read_dir(path).expect("framework source directory should be readable") {
        let path = entry
            .expect("framework source entry should be readable")
            .path();
        if path.is_dir() {
            assert_imports_only_under_any(&path, allowed_roots, modules);
            continue;
        }

        if allowed_roots.iter().any(|root| path.starts_with(root))
            || path.extension().and_then(|extension| extension.to_str()) != Some("rs")
        {
            continue;
        }

        let source = std::fs::read_to_string(&path).expect("framework source file should read");
        for module in modules {
            assert!(
                !source_imports_crate_module(&source, module),
                "{} must import crate::{} only under one of {:?}",
                path.display(),
                module,
                allowed_roots
            );
        }
    }
}

fn source_imports_crate_module(source: &str, module: &str) -> bool {
    if source.contains(&format!("crate::{module}::")) {
        return true;
    }

    source.lines().any(|line| {
        let line = line.trim();
        if line == format!("use crate::{module};")
            || line.starts_with(&format!("use crate::{module}::"))
        {
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
