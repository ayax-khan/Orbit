-- Add contacts table for user-to-user connections
CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) NOT NULL,
    contact_user_id UUID REFERENCES users(id) NOT NULL,
    display_name VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    UNIQUE(user_id, contact_user_id)
);

CREATE INDEX idx_contacts_user_id ON contacts(user_id);
CREATE INDEX idx_contacts_contact_user_id ON contacts(contact_user_id);

-- Update sessions table to support user-to-user (extension-to-extension) sessions
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS session_type VARCHAR DEFAULT 'device' NOT NULL; -- 'device' or 'user'
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS host_user_id UUID REFERENCES users(id);
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS client_device_id UUID REFERENCES devices(id);

-- Add indexes for user-to-user session queries
CREATE INDEX IF NOT EXISTS idx_sessions_host_user_id_status ON sessions(host_user_id, status) WHERE session_type = 'user';
CREATE INDEX IF NOT EXISTS idx_sessions_client_user_id_type ON sessions(client_user_id, session_type) WHERE session_type = 'user';
