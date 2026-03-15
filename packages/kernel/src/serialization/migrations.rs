use crate::error::{KernelError, KernelResult};

use super::schema::{KernelDocument, SCHEMA_VERSION};

/// Migrate a document from an older schema version to the current one.
pub fn migrate(doc: KernelDocument) -> KernelResult<KernelDocument> {
    if doc.version == SCHEMA_VERSION {
        return Ok(doc);
    }

    if doc.version > SCHEMA_VERSION {
        return Err(KernelError::Migration {
            from: doc.version,
            to: SCHEMA_VERSION,
            detail: "Document is from a newer version of blockCAD".into(),
        });
    }

    // Apply migrations sequentially
    let mut current = doc;
    while current.version < SCHEMA_VERSION {
        current = migrate_one_step(current)?;
    }
    Ok(current)
}

fn migrate_one_step(doc: KernelDocument) -> KernelResult<KernelDocument> {
    match doc.version {
        // Add migration handlers here as schema evolves:
        // 1 => migrate_v1_to_v2(doc),
        // 2 => migrate_v2_to_v3(doc),
        v => Err(KernelError::Migration {
            from: v,
            to: v + 1,
            detail: format!("No migration path from v{}", v),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_passes_through() {
        let doc = KernelDocument::new("Test".into(), vec![]);
        let result = migrate(doc).unwrap();
        assert_eq!(result.version, SCHEMA_VERSION);
    }

    #[test]
    fn future_version_rejected() {
        let doc = KernelDocument {
            schema_url: None,
            version: SCHEMA_VERSION + 1,
            metadata: super::super::schema::Metadata::default(),
            features: vec![],
        };
        assert!(migrate(doc).is_err());
    }
}
