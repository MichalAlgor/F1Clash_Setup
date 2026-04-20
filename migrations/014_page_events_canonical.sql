-- Add canonical_path (dynamic IDs replaced with :id / :hash) and
-- kind ('page' for GET navigations, 'action' for mutations).
-- Existing rows default to kind='page'; canonical_path stays NULL
-- and queries fall back to raw path via COALESCE.
ALTER TABLE page_events
    ADD COLUMN canonical_path TEXT,
    ADD COLUMN kind            TEXT NOT NULL DEFAULT 'page';
