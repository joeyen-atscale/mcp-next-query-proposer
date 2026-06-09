//! Minimal MQO structural validation.
//!
//! Validates that a proposed MQO JSON object has:
//! - `measures` array, non-empty
//! - `dimensions` array (may be empty)
//! - `model` string, non-empty
//!
//! This mirrors the structural contract in mqo-spec without importing it.

use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    MissingModel,
    EmptyModel,
    MissingMeasures,
    EmptyMeasures,
    MissingDimensions,
    InvalidMeasureEntry(usize),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingModel => write!(f, "MQO missing 'model' field"),
            ValidationError::EmptyModel => write!(f, "MQO 'model' is empty string"),
            ValidationError::MissingMeasures => write!(f, "MQO missing 'measures' array"),
            ValidationError::EmptyMeasures => write!(f, "MQO 'measures' array is empty"),
            ValidationError::MissingDimensions => write!(f, "MQO missing 'dimensions' array"),
            ValidationError::InvalidMeasureEntry(i) => {
                write!(f, "MQO measures[{i}] missing 'unique_name'")
            }
        }
    }
}

/// Returns Ok(()) if the MQO is structurally valid, Err(reason) otherwise.
pub fn validate(mqo: &Value) -> Result<(), ValidationError> {
    // model
    match mqo.get("model") {
        None => return Err(ValidationError::MissingModel),
        Some(m) => {
            if m.as_str().map(str::is_empty).unwrap_or(true) {
                return Err(ValidationError::EmptyModel);
            }
        }
    }

    // measures
    match mqo.get("measures") {
        None => return Err(ValidationError::MissingMeasures),
        Some(m) => match m.as_array() {
            None => return Err(ValidationError::MissingMeasures),
            Some(arr) => {
                if arr.is_empty() {
                    return Err(ValidationError::EmptyMeasures);
                }
                for (i, entry) in arr.iter().enumerate() {
                    if entry.get("unique_name").and_then(Value::as_str).is_none() {
                        return Err(ValidationError::InvalidMeasureEntry(i));
                    }
                }
            }
        },
    }

    // dimensions must be present (array, may be empty)
    match mqo.get("dimensions") {
        None => return Err(ValidationError::MissingDimensions),
        Some(d) if d.as_array().is_none() => return Err(ValidationError::MissingDimensions),
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn valid_mqo() {
        let mqo = json!({
            "model": "sales",
            "measures": [{"unique_name": "sales.revenue"}],
            "dimensions": []
        });
        assert!(validate(&mqo).is_ok());
    }

    #[test]
    fn missing_model() {
        let mqo = json!({
            "measures": [{"unique_name": "sales.revenue"}],
            "dimensions": []
        });
        assert_eq!(validate(&mqo), Err(ValidationError::MissingModel));
    }

    #[test]
    fn empty_measures() {
        let mqo = json!({
            "model": "sales",
            "measures": [],
            "dimensions": []
        });
        assert_eq!(validate(&mqo), Err(ValidationError::EmptyMeasures));
    }

    #[test]
    fn missing_dimensions() {
        let mqo = json!({
            "model": "sales",
            "measures": [{"unique_name": "sales.revenue"}]
        });
        assert_eq!(validate(&mqo), Err(ValidationError::MissingDimensions));
    }
}
