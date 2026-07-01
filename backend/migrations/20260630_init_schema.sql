CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR UNIQUE NOT NULL,
    password_hash VARCHAR NOT NULL,
    google_id VARCHAR,
    full_name VARCHAR NOT NULL,
    avatar_url VARCHAR,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    last_login TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT TRUE NOT NULL,
    email_verified BOOLEAN DEFAULT FALSE NOT NULL
);

CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) NOT NULL,
    device_name VARCHAR NOT NULL,
    device_type VARCHAR NOT NULL,
    os_version VARCHAR NOT NULL,
    agent_version VARCHAR NOT NULL,
    device_token VARCHAR UNIQUE NOT NULL,
    public_key TEXT NOT NULL,
    last_seen TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    ip_address INET,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    is_online BOOLEAN DEFAULT FALSE NOT NULL,
    allow_remote_access BOOLEAN DEFAULT FALSE NOT NULL
);

CREATE INDEX idx_devices_user_id_is_online ON devices(user_id, is_online);

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    host_device_id UUID REFERENCES devices(id) NOT NULL,
    client_user_id UUID REFERENCES users(id) NOT NULL,
    session_token VARCHAR UNIQUE NOT NULL,
    status VARCHAR NOT NULL, -- 'active', 'inactive', 'expired'
    started_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    ended_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    connection_quality VARCHAR,
    bytes_sent BIGINT DEFAULT 0 NOT NULL,
    bytes_received BIGINT DEFAULT 0 NOT NULL
);

CREATE INDEX idx_sessions_host_device_id_status ON sessions(host_device_id, status);
CREATE INDEX idx_sessions_client_user_id ON sessions(client_user_id);

CREATE TABLE connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id) NOT NULL,
    connection_type VARCHAR NOT NULL,
    latency_ms INTEGER NOT NULL,
    packet_loss FLOAT NOT NULL,
    bandwidth_kbps INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    disconnected_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_connections_session_id ON connections(session_id);

CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) NOT NULL,
    token_hash VARCHAR UNIQUE NOT NULL,
    device_fingerprint VARCHAR NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_refresh_tokens_user_id_expires_at ON refresh_tokens(user_id, expires_at);

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    device_id UUID REFERENCES devices(id),
    session_id UUID REFERENCES sessions(id),
    event_type VARCHAR NOT NULL,
    event_description TEXT NOT NULL,
    ip_address INET NOT NULL,
    user_agent VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_audit_logs_user_id_created_at ON audit_logs(user_id, created_at);
CREATE INDEX idx_audit_logs_device_id_created_at ON audit_logs(device_id, created_at);

CREATE TABLE user_settings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) UNIQUE NOT NULL,
    session_timeout_minutes INTEGER DEFAULT 720 NOT NULL,
    allow_remote_access_by_default BOOLEAN DEFAULT FALSE NOT NULL,
    notification_preferences JSONB DEFAULT '{}'::jsonb NOT NULL,
    theme_preference VARCHAR DEFAULT 'light' NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);
