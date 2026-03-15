use thiserror::Error;

pub type KernelResult<T> = Result<T, KernelError>;

#[derive(Error, Debug, Clone)]
pub enum KernelError {
    #[error("Geometry error: {0}")]
    Geometry(String),

    #[error("Topology error: {0}")]
    Topology(String),

    #[error("Constraint solver error: {reason}")]
    ConstraintSolver { reason: String, dof: Option<i32> },

    #[error("Operation failed: {op} — {detail}")]
    Operation { op: String, detail: String },

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Schema migration error: from v{from} to v{to}: {detail}")]
    Migration { from: u32, to: u32, detail: String },

    #[error("Invalid parameter: {param} = {value}")]
    InvalidParameter { param: String, value: String },

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Over-constrained: redundant constraint")]
    OverConstrained,

    #[error("Under-constrained: {dof} degrees of freedom remain")]
    UnderConstrained { dof: u32 },

    #[error("Internal error: {0}")]
    Internal(String),
}

impl KernelError {
    pub fn kind_str(&self) -> &'static str {
        match self {
            KernelError::Geometry(_) => "geometry",
            KernelError::Topology(_) => "topology",
            KernelError::ConstraintSolver { .. } => "constraint_solver",
            KernelError::Operation { .. } => "operation",
            KernelError::Serialization(_) => "serialization",
            KernelError::Migration { .. } => "migration",
            KernelError::InvalidParameter { .. } => "invalid_parameter",
            KernelError::NotFound(_) => "not_found",
            KernelError::OverConstrained => "over_constrained",
            KernelError::UnderConstrained { .. } => "under_constrained",
            KernelError::Internal(_) => "internal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = KernelError::Geometry("invalid curve parameter".into());
        assert_eq!(err.to_string(), "Geometry error: invalid curve parameter");
        assert_eq!(err.kind_str(), "geometry");
    }

    #[test]
    fn error_kinds_are_distinct() {
        let errors: Vec<KernelError> = vec![
            KernelError::Geometry("test".into()),
            KernelError::Topology("test".into()),
            KernelError::ConstraintSolver { reason: "test".into(), dof: None },
            KernelError::Operation { op: "test".into(), detail: "test".into() },
            KernelError::Serialization("test".into()),
            KernelError::Migration { from: 1, to: 2, detail: "test".into() },
            KernelError::InvalidParameter { param: "x".into(), value: "0".into() },
            KernelError::NotFound("test".into()),
            KernelError::OverConstrained,
            KernelError::UnderConstrained { dof: 3 },
            KernelError::Internal("test".into()),
        ];
        let kinds: Vec<&str> = errors.iter().map(|e| e.kind_str()).collect();
        let mut unique = kinds.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(kinds.len(), unique.len(), "All error kinds must be distinct");
    }
}
