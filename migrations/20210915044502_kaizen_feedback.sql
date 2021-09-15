CREATE TABLE IF NOT EXISTS kaizen_feedback (
	campaign_id uuid NOT NULL REFERENCES kaizen_campaign(uuid) ON DELETE CASCADE,
	description VARCHAR(400) UNIQUE DEFAULT NULL,
	rating BOOLEAN NOT NULL,
    time timestamptz NOT NULL,
    uuid UUID PRIMARY KEY NOT NULL UNIQUE
);
