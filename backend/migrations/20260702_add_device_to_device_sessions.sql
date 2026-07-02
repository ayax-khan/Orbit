-- Add support for device-to-device (agent-to-agent) sessions
-- This allows one desktop agent to connect to another desktop agent

-- Update sessions table to support device-to-device sessions
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS client_device_id UUID REFERENCES devices(id);

-- Add index for device-to-device session queries
CREATE INDEX IF NOT EXISTS idx_sessions_host_device_client_device ON sessions(host_device_id, client_device_id) WHERE session_type = 'device';

-- Update session_type to support 'device-to-device' 
-- Note: We'll use 'device' type but with both host_device_id and client_device_id set
