import { api } from './api';

export interface SessionResponse {
  session_id: string;
  turn_servers: string[];
  stun_servers: string[];
}

export const sessionService = {
  async createSession(hostDeviceId: string, token: string): Promise<SessionResponse> {
    return api.post<SessionResponse>('/sessions/create', { host_device_id: hostDeviceId }, token);
  },
};
