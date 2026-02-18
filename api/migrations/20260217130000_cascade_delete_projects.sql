-- Change foreign keys on survey_responses and connectors from RESTRICT to CASCADE

ALTER TABLE survey_responses
    DROP CONSTRAINT survey_responses_project_id_fkey,
    ADD CONSTRAINT survey_responses_project_id_fkey
        FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE;

ALTER TABLE connectors
    DROP CONSTRAINT connectors_project_id_fkey,
    ADD CONSTRAINT connectors_project_id_fkey
        FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE;
