use research_contracts::ReproducibilityIdentity;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvenanceError {
    Invalid(String),
    HashMismatch { expected: String, actual: String },
}

impl std::fmt::Display for ProvenanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => f.write_str(message),
            Self::HashMismatch { expected, actual } => {
                write!(
                    f,
                    "identity hash mismatch: expected {expected}, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for ProvenanceError {}

pub fn verify_identity(
    identity: &ReproducibilityIdentity,
    expected_hash: Option<&str>,
) -> Result<String, ProvenanceError> {
    let actual = identity
        .identity_hash()
        .map_err(|error| ProvenanceError::Invalid(error.to_string()))?;
    if let Some(expected) = expected_hash {
        if expected != actual {
            return Err(ProvenanceError::HashMismatch {
                expected: expected.to_owned(),
                actual,
            });
        }
    }
    Ok(actual)
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> ReproducibilityIdentity {
        ReproducibilityIdentity {
            source_identity: "source-v1".into(),
            binary_identity: "binary-v1".into(),
            configuration_identity: "config-v1".into(),
            parameter_identity: "params-v1".into(),
            seed_identity: "seed-1".into(),
            output_identity: "output-v1".into(),
            user_identities: vec!["instrument-x".into()],
        }
    }

    #[test]
    fn identity_hash_is_repeatable_and_checks_expected_value() {
        let value = identity();
        let hash = verify_identity(&value, None).expect("valid identity");
        assert_eq!(verify_identity(&value, Some(&hash)).unwrap(), hash);
        assert!(matches!(
            verify_identity(&value, Some("wrong")),
            Err(ProvenanceError::HashMismatch { .. })
        ));
    }

    #[test]
    fn byte_hash_is_stable() {
        assert_eq!(hash_bytes(b"trinity"), hash_bytes(b"trinity"));
        assert_ne!(hash_bytes(b"trinity"), hash_bytes(b"other"));
    }
}
