//! Shared helpers for honest, deterministic JSON Schema roots.
//!
//! Runtime custom Deserialize/validate remains authoritative. Extensions listed
//! under `x-gzmo-runtime-validation` name cross-field checks JSON Schema cannot
//! express. Schemas never claim to verify signatures, digest bindings, or event
//! hashes by themselves.

use schemars::schema::RootSchema;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

/// Recursively sort object keys; preserve array order.
pub fn recursively_sort_object_keys(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut sorted = Map::with_capacity(entries.len());
            for (k, v) in entries {
                sorted.insert(k, recursively_sort_object_keys(v));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .map(recursively_sort_object_keys)
                .collect(),
        ),
        other => other,
    }
}

/// Attach root identity + honest runtime-validation extension list.
pub fn seal_root_schema(
    mut root: RootSchema,
    id_uri: &str,
    title: &str,
    schema_id: &str,
    runtime_validation: &[&str],
) -> RootSchema {
    {
        let meta = root.schema.metadata.get_or_insert_with(Default::default);
        meta.id = Some(id_uri.to_owned());
        meta.title = Some(title.to_owned());
    }

    // Frozen contract surface denies unknown top-level properties.
    let object = root.schema.object.get_or_insert_with(Default::default);
    object.additional_properties = Some(Box::new(false.into()));

    root.schema.extensions.insert(
        "x-gzmo-schema-id".to_owned(),
        Value::String(schema_id.to_owned()),
    );
    root.schema.extensions.insert(
        "x-gzmo-runtime-validation".to_owned(),
        json!(runtime_validation),
    );
    root
}

/// Pretty-print sealed schema with sorted keys and a trailing newline; atomic replace.
pub fn write_schema_value(path: &Path, root: RootSchema) -> Result<(), Box<dyn std::error::Error>> {
    let value = serde_json::to_value(root)?;
    let sorted = recursively_sort_object_keys(value);
    let text = serde_json::to_string_pretty(&sorted)? + "\n";
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("json.tmp");
    let write_result = (|| -> Result<(), Box<dyn std::error::Error>> {
        fs::write(&temp, &text)?;
        fs::rename(&temp, path)?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    write_result
}
