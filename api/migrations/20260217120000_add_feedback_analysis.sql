CREATE TABLE feedback_analyses (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    response_count INTEGER NOT NULL,
    analysis JSONB NOT NULL,
    narrative TEXT NOT NULL,
    model_used VARCHAR(100) NOT NULL,
    input_tokens INTEGER,
    output_tokens INTEGER,
    duration_ms INTEGER
);

CREATE INDEX idx_feedback_analyses_project_id ON feedback_analyses(project_id);
