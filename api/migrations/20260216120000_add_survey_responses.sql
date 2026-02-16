CREATE TABLE survey_responses (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    date TIMESTAMP,
    country VARCHAR(255),
    url TEXT,
    device VARCHAR(255),
    browser VARCHAR(255),
    os VARCHAR(255),
    ratings DOUBLE PRECISION,
    comments TEXT,
    raw JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_survey_responses_project_id ON survey_responses(project_id);
CREATE INDEX idx_survey_responses_date ON survey_responses(date);
