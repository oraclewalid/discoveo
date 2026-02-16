use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// EmbeddingService manages the FastEmbed model and generates embeddings
/// Pattern: Singleton model instance, shared across requests
#[derive(Clone)]
pub struct EmbeddingService {
    model: Arc<TextEmbedding>,
}

impl EmbeddingService {
    /// Initialize the embedding model (one-time at startup)
    /// Model: MultilingualE5Base (768 dimensions, optimized for French and 50+ languages)
    pub fn new() -> Result<Self, String> {
        info!("Initializing FastEmbed model (MultilingualE5Base - French optimized)");

        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::MultilingualE5Base).with_show_download_progress(true),
        )
        .map_err(|e| {
            error!(error = %e, "Failed to initialize FastEmbed model");
            format!("Failed to initialize embedding model: {}", e)
        })?;

        info!("FastEmbed MultilingualE5Base model loaded successfully");
        Ok(Self {
            model: Arc::new(model),
        })
    }

    /// Generate embeddings for a batch of texts
    /// Returns Vec of embeddings in the same order as input
    /// Empty/whitespace strings return None
    pub fn generate_embeddings(&self, texts: Vec<String>) -> Result<Vec<Option<Vec<f32>>>, String> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        debug!(count = texts.len(), "Generating embeddings");

        // Filter out empty texts but track their indices
        let mut valid_indices = Vec::new();
        let mut valid_texts = Vec::new();

        for (idx, text) in texts.iter().enumerate() {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                valid_indices.push(idx);
                valid_texts.push(trimmed.to_string());
            }
        }

        if valid_texts.is_empty() {
            warn!("All input texts are empty, skipping embedding generation");
            return Ok(vec![None; texts.len()]);
        }

        // Generate embeddings for valid texts
        let embeddings = self
            .model
            .embed(valid_texts, None)
            .map_err(|e| {
                error!(error = %e, "Failed to generate embeddings");
                format!("Embedding generation failed: {}", e)
            })?;

        // Map embeddings back to original indices
        let mut result = vec![None; texts.len()];
        for (valid_idx, embedding) in embeddings.into_iter().enumerate() {
            let original_idx = valid_indices[valid_idx];
            result[original_idx] = Some(embedding);
        }

        info!(
            total = texts.len(),
            valid = valid_indices.len(),
            "Embeddings generated"
        );

        Ok(result)
    }

    /// Generate single embedding (convenience wrapper)
    pub fn generate_embedding(&self, text: &str) -> Result<Option<Vec<f32>>, String> {
        let results = self.generate_embeddings(vec![text.to_string()])?;
        Ok(results.into_iter().next().flatten())
    }
}

/// Background job to generate embeddings for pending survey responses
pub async fn generate_embeddings_for_project(
    project_id: Uuid,
    embedding_service: EmbeddingService,
    survey_repo: crate::infrastructure::survey_repository::SurveyRepository,
) {
    info!(
        project_id = %project_id,
        "Starting background embedding generation"
    );

    // Fetch responses with pending embeddings
    let responses = match survey_repo.find_pending_embeddings(project_id).await {
        Ok(r) => r,
        Err(e) => {
            error!(
                project_id = %project_id,
                error = %e,
                "Failed to fetch pending embeddings"
            );
            return;
        }
    };

    if responses.is_empty() {
        info!(project_id = %project_id, "No pending embeddings");
        return;
    }

    info!(
        project_id = %project_id,
        count = responses.len(),
        "Processing pending embeddings"
    );

    // Batch process: group responses and their comments
    let mut response_ids = Vec::new();
    let mut comments = Vec::new();

    for response in responses {
        response_ids.push(response.id);
        comments.push(response.comments.unwrap_or_default());
    }

    // Generate embeddings in batch
    let embeddings = match embedding_service.generate_embeddings(comments) {
        Ok(e) => e,
        Err(e) => {
            error!(
                project_id = %project_id,
                error = %e,
                "Failed to generate embeddings batch"
            );
            // Mark all as failed
            for id in response_ids {
                let _ = survey_repo.update_embedding_status(id, "failed").await;
            }
            return;
        }
    };

    // Update each response with its embedding
    let mut success_count = 0;
    let mut skip_count = 0;
    let mut fail_count = 0;

    for (idx, response_id) in response_ids.iter().enumerate() {
        let embedding = &embeddings[idx];

        let result = if let Some(emb) = embedding {
            survey_repo
                .update_embedding(*response_id, emb.clone())
                .await
        } else {
            // Empty comment, mark as skipped
            survey_repo
                .update_embedding_status(*response_id, "skipped")
                .await
        };

        match result {
            Ok(_) => {
                if embedding.is_some() {
                    success_count += 1;
                } else {
                    skip_count += 1;
                }
            }
            Err(e) => {
                error!(
                    response_id = %response_id,
                    error = %e,
                    "Failed to update embedding"
                );
                fail_count += 1;
            }
        }
    }

    info!(
        project_id = %project_id,
        success = success_count,
        skipped = skip_count,
        failed = fail_count,
        "Embedding generation completed"
    );
}
