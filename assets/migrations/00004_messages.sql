ALTER TABLE attachments
  ADD COLUMN deleted BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE messages
  ADD COLUMN attachments JSONB NOT NULL DEFAULT '[]'::JSONB;