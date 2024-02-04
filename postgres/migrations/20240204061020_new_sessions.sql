-- Add migration script here

CREATE TABLE IF NOT EXISTS sessions (
    session_id UUID PRIMARY KEY not null,
    session_chain_id UUID not null,
    service_id UUID not null,
    created_at TIMESTAMPTZ not null
)

CREATE TABLE IF NOT EXISTS session_chains (
    session_chain_id UUID PRIMARY KEY not null,
    service_id UUID not null,
)

CREATE TABLE IF NOT EXISTS services (
    service_id UUID PRIMARY KEY not null,
    config JSON
)
