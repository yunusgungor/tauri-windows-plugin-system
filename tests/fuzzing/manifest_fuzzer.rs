//! Fuzzing tests for plugin manifest validation

use crate::plugin_loader::manifest::{PluginManifest, ManifestError};
use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Arbitrary)]
struct FuzzManifestInput {
    name_present: bool,
    version_present: bool,
    description_present: bool,
    author_present: bool,
    permissions_present: bool,
    min_host_version_present: bool,
    entry_point_present: bool,
    
    // Control malformed fields
    malformed_name: bool,
    malformed_version: bool,
    malformed_description: bool,
    malformed_author: bool,
    malformed_permissions: bool,
    malformed_min_host_version: bool,
    malformed_entry_point: bool,
    
    // Random strings to use
    name: String,
    version: String,
    description: String,
    author: String,
    permissions: Vec<String>,
    min_host_version: String,
    entry_point: String,
    
    // Extra random fields
    extra_fields: HashMap<String, String>,
}

/// Fuzz target for manifest validation
fuzz_target!(|input: FuzzManifestInput| {
    // Create a JSON manifest based on the fuzz input
    let mut manifest_value = json!({});
    
    // Add fields based on presence flags
    if input.name_present {
        let name = if input.malformed_name {
            // Use potentially problematic strings or types
            json!([1, 2, 3]) // Array instead of string
        } else {
            json!(input.name)
        };
        manifest_value["name"] = name;
    }
    
    if input.version_present {
        let version = if input.malformed_version {
            // Invalid version format
            json!("invalid_version_format")
        } else {
            json!(input.version)
        };
        manifest_value["version"] = version;
    }
    
    if input.description_present {
        let description = if input.malformed_description {
            json!({})
        } else {
            json!(input.description)
        };
        manifest_value["description"] = description;
    }
    
    if input.author_present {
        let author = if input.malformed_author {
            json!(null)
        } else {
            json!(input.author)
        };
        manifest_value["author"] = author;
    }
    
    if input.permissions_present {
        let permissions = if input.malformed_permissions {
            json!(input.permissions.join(","))
        } else {
            json!(input.permissions)
        };
        manifest_value["permissions"] = permissions;
    }
    
    if input.min_host_version_present {
        let min_host_version = if input.malformed_min_host_version {
            json!("bad-version!")
        } else {
            json!(input.min_host_version)
        };
        manifest_value["min_host_version"] = min_host_version;
    }
    
    if input.entry_point_present {
        let entry_point = if input.malformed_entry_point {
            json!({})
        } else {
            json!(input.entry_point)
        };
        manifest_value["entry_point"] = entry_point;
    }
    
    // Add random extra fields
    for (key, value) in &input.extra_fields {
        manifest_value[key] = json!(value);
    }
    
    // Convert to string
    let manifest_json = serde_json::to_string(&manifest_value).unwrap_or_else(|_| "{}".to_string());
    
    // Test parsing the manifest
    let result = PluginManifest::from_json(&manifest_json);
    
    // We don't assert anything here - fuzzing just looks for panics
    // But we can log the result for analysis
    match &result {
        Ok(manifest) => {
            // Successfully parsed manifest
        },
        Err(err) => {
            // Error parsing manifest
            match err {
                ManifestError::MissingField(_) => {},
                ManifestError::InvalidJson(_) => {},
                ManifestError::InvalidPermission(_) => {},
                ManifestError::InvalidVersionFormat(_) => {},
                _ => {},
            }
        }
    }
    
    // If we successfully parsed the manifest, also test permission validation
    if let Ok(manifest) = result {
        let _perm_result = manifest.validate_permissions();
        // Again, we don't assert anything, just making sure it doesn't panic
    }
});
