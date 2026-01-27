use arrow_array::{
    ArrayRef, FixedSizeListArray, Float32Array, RecordBatch, StringArray, TimestampMillisecondArray,
};
use arrow_schema::{DataType, Field, Schema, TimeUnit};
use std::sync::Arc;

const EMBEDDING_DIMENSION: i32 = 384;

/// Create the Arrow schema for artifacts
pub fn create_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("type", DataType::Utf8, false),
        Field::new("name", DataType::Utf8, true),
        Field::new("content", DataType::Utf8, false),
        Field::new("format", DataType::Utf8, false),
        Field::new("metadata", DataType::Utf8, false), // JSON string
        Field::new(
            "embedding",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                EMBEDDING_DIMENSION,
            ),
            false,
        ),
        Field::new("embedding_model", DataType::Utf8, false),
        Field::new(
            "created_at",
            DataType::Timestamp(TimeUnit::Millisecond, None),
            false,
        ),
        Field::new(
            "updated_at",
            DataType::Timestamp(TimeUnit::Millisecond, None),
            false,
        ),
    ]))
}

/// Convert artifacts to Arrow RecordBatch
pub fn artifacts_to_batch(artifacts: &[crate::services::Artifact]) -> anyhow::Result<RecordBatch> {
    let schema = create_schema();

    let ids: ArrayRef = Arc::new(StringArray::from(
        artifacts.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
    ));

    let types: ArrayRef = Arc::new(StringArray::from(
        artifacts
            .iter()
            .map(|a| a.artifact_type.to_string())
            .collect::<Vec<_>>(),
    ));

    let names: ArrayRef = Arc::new(StringArray::from(
        artifacts
            .iter()
            .map(|a| a.name.as_deref())
            .collect::<Vec<_>>(),
    ));

    let contents: ArrayRef = Arc::new(StringArray::from(
        artifacts
            .iter()
            .map(|a| a.content.as_str())
            .collect::<Vec<_>>(),
    ));

    let formats: ArrayRef = Arc::new(StringArray::from(
        artifacts
            .iter()
            .map(|a| a.format.to_string())
            .collect::<Vec<_>>(),
    ));

    let metadata: ArrayRef = Arc::new(StringArray::from(
        artifacts
            .iter()
            .map(|a| serde_json::to_string(&a.metadata).unwrap_or_default())
            .collect::<Vec<_>>(),
    ));

    // Build FixedSizeList for embeddings
    let embeddings: Vec<f32> = artifacts
        .iter()
        .flat_map(|a| {
            a.embedding
                .as_ref()
                .map(|e| e.as_slice())
                .unwrap_or(&[0.0; 384])
        })
        .copied()
        .collect();
    let values = Float32Array::from(embeddings);
    let field = Arc::new(Field::new("item", DataType::Float32, true));
    let embeddings_array: ArrayRef = Arc::new(
        FixedSizeListArray::try_new(field, EMBEDDING_DIMENSION, Arc::new(values), None)
            .map_err(|e| anyhow::anyhow!("Failed to create embeddings array: {}", e))?,
    );

    let embedding_models: ArrayRef = Arc::new(StringArray::from(
        artifacts
            .iter()
            .map(|a| a.embedding_model.as_str())
            .collect::<Vec<_>>(),
    ));

    let created_ats: ArrayRef = Arc::new(TimestampMillisecondArray::from(
        artifacts
            .iter()
            .map(|a| a.created_at.timestamp_millis())
            .collect::<Vec<_>>(),
    ));

    let updated_ats: ArrayRef = Arc::new(TimestampMillisecondArray::from(
        artifacts
            .iter()
            .map(|a| a.updated_at.timestamp_millis())
            .collect::<Vec<_>>(),
    ));

    RecordBatch::try_new(
        schema,
        vec![
            ids,
            types,
            names,
            contents,
            formats,
            metadata,
            embeddings_array,
            embedding_models,
            created_ats,
            updated_ats,
        ],
    )
    .map_err(|e| anyhow::anyhow!("Failed to create record batch: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::{Artifact, ArtifactType, ContentFormat};
    use std::collections::HashMap;

    #[test]
    fn schema_has_required_fields() {
        let schema = create_schema();
        let field_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

        assert!(field_names.contains(&"id"));
        assert!(field_names.contains(&"type"));
        assert!(field_names.contains(&"name"));
        assert!(field_names.contains(&"content"));
        assert!(field_names.contains(&"format"));
        assert!(field_names.contains(&"metadata"));
        assert!(field_names.contains(&"embedding"));
        assert!(field_names.contains(&"embedding_model"));
        assert!(field_names.contains(&"created_at"));
        assert!(field_names.contains(&"updated_at"));
    }

    #[test]
    fn schema_field_count() {
        let schema = create_schema();
        assert_eq!(schema.fields().len(), 10);
    }

    #[test]
    fn schema_id_is_not_nullable() {
        let schema = create_schema();
        let id_field = schema.field_with_name("id").unwrap();
        assert!(!id_field.is_nullable());
    }

    #[test]
    fn schema_name_is_nullable() {
        let schema = create_schema();
        let name_field = schema.field_with_name("name").unwrap();
        assert!(name_field.is_nullable());
    }

    #[test]
    fn schema_embedding_is_fixed_size_384() {
        let schema = create_schema();
        let embedding_field = schema.field_with_name("embedding").unwrap();
        if let DataType::FixedSizeList(_, size) = embedding_field.data_type() {
            assert_eq!(*size, 384);
        } else {
            panic!("Expected FixedSizeList for embedding field");
        }
    }

    #[test]
    fn artifacts_to_batch_single_artifact() {
        let mut artifact = Artifact::new(
            ArtifactType::Intent,
            "Test content".to_string(),
            ContentFormat::Markdown,
            Some("test-name".to_string()),
            HashMap::new(),
            "test-model".to_string(),
        );
        artifact.embedding = Some(vec![0.0; 384]);

        let batch = artifacts_to_batch(&[artifact]).unwrap();
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 10);
    }

    #[test]
    fn artifacts_to_batch_multiple_artifacts() {
        let artifacts: Vec<Artifact> = (0..5)
            .map(|i| {
                let mut a = Artifact::new(
                    ArtifactType::Intent,
                    format!("Content {}", i),
                    ContentFormat::Markdown,
                    None,
                    HashMap::new(),
                    "model".to_string(),
                );
                a.embedding = Some(vec![i as f32; 384]);
                a
            })
            .collect();

        let batch = artifacts_to_batch(&artifacts).unwrap();
        assert_eq!(batch.num_rows(), 5);
    }

    #[test]
    fn artifacts_to_batch_empty_vec() {
        let artifacts: Vec<Artifact> = vec![];
        let batch = artifacts_to_batch(&artifacts).unwrap();
        assert_eq!(batch.num_rows(), 0);
    }

    #[test]
    fn artifacts_to_batch_preserves_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());

        let mut artifact = Artifact::new(
            ArtifactType::Contract,
            "Test".to_string(),
            ContentFormat::Json,
            None,
            metadata,
            "model".to_string(),
        );
        artifact.embedding = Some(vec![0.0; 384]);

        let batch = artifacts_to_batch(&[artifact]).unwrap();

        let metadata_col = batch.column(5);
        let metadata_array = metadata_col.as_any().downcast_ref::<StringArray>().unwrap();
        let metadata_json = metadata_array.value(0);
        assert!(metadata_json.contains("key"));
        assert!(metadata_json.contains("value"));
    }
}
