CREATE TABLE IF NOT EXISTS kaizen_feedbacks (
	campaign_id uuid NOT NULL REFERENCES kaizen_campaign(uuid) ON DELETE CASCADE,
	description VARCHAR(400) DEFAULT NULL,
	page_url INTEGER NOT NULL REFERENCES kaizen_campaign_pages(ID) ON DELETE CASCADE,
	helpful BOOLEAN NOT NULL,
    time timestamptz NOT NULL,
    uuid UUID PRIMARY KEY NOT NULL UNIQUE
);
