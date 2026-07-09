use crate::render::silhouette;

const SOURCE: &str = include_str!("../filter.wgsl");

pub(super) fn module_source() -> String {
    silhouette::wgsl_module_source(SOURCE)
}

#[cfg(test)]
pub(super) fn raw_source() -> &'static str {
    SOURCE
}
