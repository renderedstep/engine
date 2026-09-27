//! `Eval::RequestIdentity`: the canonical form of a set of model requests
//! and the short digest that names it, so two engines that build the same
//! requests give them the same name.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

/// `Eval::RequestIdentity::VERSION`.
pub const VERSION: u64 = 1;

/// Every object's keys sorted, recursively; arrays keep their order.
pub fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(fields) => {
            let mut keys: Vec<&String> = fields.keys().collect();
            keys.sort();
            let sorted: Map<String, Value> = keys
                .into_iter()
                .map(|key| (key.clone(), canonical(&fields[key])))
                .collect();
            Value::Object(sorted)
        }
        Value::Array(parts) => Value::Array(parts.iter().map(canonical).collect()),
        other => other.clone(),
    }
}

/// The canonical form as Ruby's `JSON.generate` writes it: no spaces,
/// non-ASCII as itself, "/" unescaped, control characters as `\uXXXX`
/// unless they have a short escape.
pub fn canonical_text(requests: &Value) -> String {
    serde_json::to_string(&canonical(requests)).expect("a JSON value")
}

/// The first 16 hex characters of the canonical form's SHA-256.
pub fn digest(requests: &Value) -> String {
    let hash = Sha256::digest(canonical_text(requests).as_bytes());
    hash.iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()[..16]
        .to_string()
}

/// `Eval::RequestIdentity.of`: `{version, digest}`.
pub fn of(requests: &Value) -> Value {
    serde_json::json!({ "version": VERSION, "digest": digest(requests) })
}

/// `Eval::Classifier::Version.identity`: the classifier set's identity, at
/// the version that names its physical tokens by their bindings.
pub fn classifier_identity(requests: &Value) -> Value {
    serde_json::json!({ "version": 2, "digest": digest(requests), "scope": "known_physical_token_bindings" })
}
