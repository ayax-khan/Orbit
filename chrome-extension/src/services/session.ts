import { api } from './api';

export interface SessionResponse {
  session_id: string;
  session_type: string;
  turn_servers: string[];
  stun_servers: string[];
}

export interface Device {
  id: string;
  name: string;
  status: string;
  last_seen: string;
}

export const sessionService = {
  async createSession(hostDeviceId: string, token: string): Promise<SessionResponse> {
    return api.post<SessionResponse>('/sessions/create', { host_device_id: hostDeviceId, session_type: 'device' }, token);
  },

  async createUserSession(hostUserId: string, token: string): Promise<SessionResponse> {
    return api.post<SessionResponse>('/sessions/create', { host_user_id: hostUserId, session_type: 'user' }, token);
  },

  async endSession(sessionId: string, token: string): Promise<void> {
    await api.delete(`/sessions/${sessionId}`, token);
  },
};

export const deviceService = {
  async listDevices(token: string): Promise<Device[]> {
    return api.get<Device[]>('/devices', token);
  },

  async registerDevice(deviceName: string, token: string): Promise<{ device_id: string; device_token: string }> {
    return api.post<{ device_id: string; device_token: string }>(
      '/devices/register',
      {
        device_name: deviceName,
        os_version: 'Chrome Extension',
        public_key: 'extension-key-placeholder',
        device_type: 'extension',
        agent_version: '1.0.0',
      },
      token
    );
  },

  async createDeviceToDeviceSession(hostDeviceId: string, clientDeviceId: string, token: string): Promise<SessionResponse> {
    return api.post<SessionResponse>('/sessions/create', {
      host_device_id: hostDeviceId,
      client_device_id: clientDeviceId,
      session_type: 'device'
    }, token);
  },
};
