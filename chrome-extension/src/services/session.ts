import { api } from './api';

export interface SessionResponse {
  session_id: string;
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
    return api.post<SessionResponse>('/sessions/create', { host_device_id: hostDeviceId }, token);
  },

  async endSession(sessionId: string, token: string): Promise<void> {
    await api.post(`/sessions/${sessionId}`, {}, token);
  },
};

export const deviceService = {
  async listDevices(token: string): Promise<Device[]> {
    return api.get<Device[]>('/devices', token);
  },
};
