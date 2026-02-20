CREATE TABLE cro_reports (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    connector_id UUID NOT NULL REFERENCES connectors(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    executive_summary TEXT NOT NULL,
    funnel_analysis JSONB NOT NULL,
    qualitative_insights JSONB NOT NULL,
    recommendations JSONB NOT NULL,
    model_used VARCHAR(100) NOT NULL,
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    tool_calls_count INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_cro_reports_project_id ON cro_reports(project_id);
