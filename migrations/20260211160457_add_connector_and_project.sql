-- Add project table
CREATE TABLE projects (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT
);

-- Add connector table with project reference
CREATE TABLE connectors (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(255) NOT NULL,
    config JSONB NOT NULL
);

-- Create index for faster lookups by project
CREATE INDEX idx_connectors_project_id ON connectors(project_id);
