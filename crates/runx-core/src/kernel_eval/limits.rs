use runx_contracts::JsonValue;

use super::KernelEvalError;

pub(super) const MAX_KERNEL_EVAL_DOCUMENT_BYTES: usize = 1024 * 1024;
pub(super) const MAX_KERNEL_EVAL_JSON_DEPTH: usize = 64;
const MAX_KERNEL_EVAL_JSON_NODES: usize = 20_000;
const MAX_KERNEL_EVAL_ARRAY_ITEMS: usize = 4_096;
const MAX_KERNEL_EVAL_OBJECT_FIELDS: usize = 512;
const MAX_KERNEL_EVAL_OBJECT_KEY_BYTES: usize = 1024;
const MAX_KERNEL_EVAL_STRING_BYTES: usize = 64 * 1024;

pub(super) fn validate_kernel_source_size(source: &str) -> Result<(), KernelEvalError> {
    if source.len() > MAX_KERNEL_EVAL_DOCUMENT_BYTES {
        return Err(KernelEvalError::InvalidInput(format!(
            "kernel eval input exceeds {MAX_KERNEL_EVAL_DOCUMENT_BYTES} bytes"
        )));
    }
    Ok(())
}

pub(super) fn validate_kernel_document_shape(document: &JsonValue) -> Result<(), KernelEvalError> {
    let mut node_count = 0usize;
    let mut pending = vec![(document, 1usize)];

    while let Some((value, depth)) = pending.pop() {
        node_count += 1;
        if node_count > MAX_KERNEL_EVAL_JSON_NODES {
            return Err(KernelEvalError::InvalidInput(format!(
                "kernel eval input exceeds {MAX_KERNEL_EVAL_JSON_NODES} JSON nodes"
            )));
        }
        if depth > MAX_KERNEL_EVAL_JSON_DEPTH {
            return Err(KernelEvalError::InvalidInput(format!(
                "kernel eval input exceeds JSON depth {MAX_KERNEL_EVAL_JSON_DEPTH}"
            )));
        }

        match value {
            JsonValue::Array(values) => {
                if values.len() > MAX_KERNEL_EVAL_ARRAY_ITEMS {
                    return Err(KernelEvalError::InvalidInput(format!(
                        "kernel eval input array exceeds {MAX_KERNEL_EVAL_ARRAY_ITEMS} items"
                    )));
                }
                for child in values {
                    pending.push((child, depth + 1));
                }
            }
            JsonValue::Object(fields) => {
                if fields.len() > MAX_KERNEL_EVAL_OBJECT_FIELDS {
                    return Err(KernelEvalError::InvalidInput(format!(
                        "kernel eval input object exceeds {MAX_KERNEL_EVAL_OBJECT_FIELDS} fields"
                    )));
                }
                for (key, child) in fields {
                    if key.len() > MAX_KERNEL_EVAL_OBJECT_KEY_BYTES {
                        return Err(KernelEvalError::InvalidInput(format!(
                            "kernel eval input object key exceeds {MAX_KERNEL_EVAL_OBJECT_KEY_BYTES} bytes"
                        )));
                    }
                    pending.push((child, depth + 1));
                }
            }
            JsonValue::String(value) => {
                if value.len() > MAX_KERNEL_EVAL_STRING_BYTES {
                    return Err(KernelEvalError::InvalidInput(format!(
                        "kernel eval input string exceeds {MAX_KERNEL_EVAL_STRING_BYTES} bytes"
                    )));
                }
            }
            JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) => {}
        }
    }

    Ok(())
}
