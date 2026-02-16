-- Enable pgvector extension for vector embeddings
CREATE EXTENSION IF NOT EXISTS vector;

-- Add embedding column (768 dimensions for MultilingualE5Base)
ALTER TABLE survey_responses
  ADD COLUMN comment_embedding vector(768),
  ADD COLUMN embedding_status VARCHAR(20) DEFAULT 'pending',
  ADD COLUMN embedding_generated_at TIMESTAMP;

-- Index for cosine similarity search (using <=> operator)
CREATE INDEX idx_survey_responses_embedding
  ON survey_responses
  USING ivfflat (comment_embedding vector_cosine_ops)
  WITH (lists = 100);

-- Index for querying pending embeddings
CREATE INDEX idx_survey_responses_embedding_status
  ON survey_responses(embedding_status)
  WHERE embedding_status = 'pending';

-- Add comments for documentation
COMMENT ON COLUMN survey_responses.comment_embedding IS 'Vector embedding of the comment text (768 dimensions, FastEmbed MultilingualE5Base - optimized for French)';
COMMENT ON COLUMN survey_responses.embedding_status IS 'Status: pending, completed, failed, skipped (null comment)';
COMMENT ON COLUMN survey_responses.embedding_generated_at IS 'Timestamp when embedding was generated';
