CREATE TABLE IF NOT EXISTS kaizen_feedback (
	campaign_id uuid NOT NULL REFERENCES kaizen_campaign(uuid) ON DELETE CASCADE,
	description VARCHAR(400) UNIQUE DEFAULT NULL,
	page_url VARCHAR(2048) UNIQUE DEFAULT NULL,
	helpful BOOLEAN NOT NULL,
    time timestamptz NOT NULL,
    uuid UUID PRIMARY KEY NOT NULL UNIQUE
);
