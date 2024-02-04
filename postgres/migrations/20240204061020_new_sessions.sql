-- Add migration script here

CREATE TABLE IF NOT EXISTS sessions (
    session_id UUID PRIMARY KEY not null,
    session_chain_id UUID not null,
    created_at TIMESTAMPTZ not null
);
