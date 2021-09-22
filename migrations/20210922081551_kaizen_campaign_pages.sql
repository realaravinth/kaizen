CREATE TABLE IF NOT EXISTS kaizen_campaign_pages (
	campaign_id uuid NOT NULL REFERENCES kaizen_campaign(uuid) ON DELETE CASCADE,
	page_url VARCHAR(2048) UNIQUE NOT NULL,
	ID SERIAL PRIMARY KEY NOT NULL
);
