use std::collections::BTreeMap;

use utoipa::openapi::{
    self, ComponentsBuilder, InfoBuilder, OpenApiBuilder, PathsBuilder, RefOr,
    path::{HttpMethod, Operation, PathItemBuilder},
};

/// Metadata needed to build an OpenAPI document.
#[derive(Clone, Debug)]
pub struct DocInfo {
    pub title: &'static str,
    pub version: &'static str,
    pub description: Option<&'static str>,
}

impl Default for DocInfo {
    fn default() -> Self {
        Self {
            title: "dy-rs API",
            version: "0.1.0",
            description: Some("API built with dy-rs"),
        }
    }
}

/// Represents a single documented endpoint gathered from `#[dy_api]`.
pub struct AutoOperation {
    pub path: &'static str,
    pub method: HttpMethod,
    pub operation: fn() -> Operation,
    pub register_schemas: fn(&mut Vec<(String, RefOr<openapi::schema::Schema>)>),
}

// Collect all documented routes from `#[dy_api]` attributes.
inventory::collect!(AutoOperation);

/// Build an OpenAPI document from all routes annotated with `#[dy_api]`.
pub fn build_auto_openapi(info: DocInfo) -> openapi::OpenApi {
    let mut path_items: BTreeMap<String, PathItemBuilder> = BTreeMap::new();

    for entry in inventory::iter::<AutoOperation>() {
        let builder = path_items
            .entry(entry.path.to_string())
            .or_insert_with(PathItemBuilder::new);

        let updated = std::mem::replace(builder, PathItemBuilder::new())
            .operation(entry.method.clone(), (entry.operation)());
        *builder = updated;
    }

    let mut paths = PathsBuilder::new();
    for (path, item) in path_items {
        paths = paths.path(path, item.build());
    }

    let mut schemas = Vec::new();
    for entry in inventory::iter::<AutoOperation>() {
        (entry.register_schemas)(&mut schemas);
    }

    let mut components_builder = ComponentsBuilder::new();
    for (name, schema) in schemas {
        components_builder = components_builder.schema(name, schema);
    }
    let components = components_builder.build();

    let mut info_builder = InfoBuilder::new().title(info.title).version(info.version);
    if let Some(description) = info.description {
        info_builder = info_builder.description(Some(description));
    }

    let mut builder = OpenApiBuilder::new()
        .info(info_builder.build())
        .paths(paths.build());

    // Only attach components if we actually collected schemas.
    if !components.schemas.is_empty() {
        builder = builder.components(Some(components));
    }

    builder.build()
}

/// Returns true if any routes have been documented via `#[dy_api]`.
pub fn has_auto_operations() -> bool {
    inventory::iter::<AutoOperation>()
        .into_iter()
        .next()
        .is_some()
}

// Re-export inventory so the macro expansion can reference it without adding
// an explicit dependency in downstream crates.
pub use inventory;
