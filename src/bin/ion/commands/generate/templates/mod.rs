// This module includes constants that can be used to render templates for generating code.
// Currently, there is no other way to add resources like `.templ` files in cargo binary crate.
// Using these constants allows the binary to access templates through these constants.

const TEMPLATE_BASE_PATH: &str = "/src/bin/ion/commands/generate/templates";

/// Represents java template constants
pub(crate) mod java {
    use super::TEMPLATE_BASE_PATH;
    pub(crate) const CLASS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "java/class.templ"
    ));
    pub(crate) const SCALAR: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "java/scalar.templ"
    ));
    pub(crate) const SEQUENCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "/java/sequence.templ"
    ));
    pub(crate) const UTIL_MACROS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "/java/util_macros.templ"
    ));
    pub(crate) const NESTED_TYPE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "/java/nested_type.templ"
    ));
}

/// Represents rust template constants
pub(crate) mod rust {
    use super::TEMPLATE_BASE_PATH;
    pub(crate) const STRUCT: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "/rust/struct.templ"
    ));
    pub(crate) const SCALAR: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "/rust/scalar.templ"
    ));
    pub(crate) const SEQUENCE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "/rust/sequence.templ"
    ));
    pub(crate) const UTIL_MACROS: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "/rust/util_macros.templ"
    ));
    pub(crate) const RESULT: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "/rust/result.templ"
    ));
    pub(crate) const NESTED_TYPE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "/rust/nested_type.templ"
    ));
    pub(crate) const IMPORT: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        TEMPLATE_BASE_PATH,
        "/rust/import.templ"
    ));
}
