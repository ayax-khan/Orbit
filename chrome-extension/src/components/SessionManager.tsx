import React, { useState } from 'react';
import { useAuthStore } from '../stores/authStore';
import { sessionService } from '../services/session';
import { RemoteScreen } from './RemoteScreen';

interface Device {
  id: string;
  name: string;
  status: string;
}

export const SessionManager: React.FC = () => {
  const token = useAuthStore((state) => state.token);
  const [devices] = useState<Device[]>([{ id: '1', name: 'My PC', status: 'online' }]); // Dummy data
  const [sessionId, setSessionId] = useState<string | null>(null);

  const handleConnect = async (deviceId: string) => {
    if (!token) return;
    try {
      const response = await sessionService.createSession(deviceId, token);
      setSessionId(response.session_id);
    } catch (error) {
      console.error('Failed to create session:', error);
    }
  };

  if (sessionId) {
    return <RemoteScreen signalingUrl={`ws://localhost:8080/ws/${sessionId}`} />;
  }

  return (
    <div className="p-4">
      <h2 className="text-xl font-bold mb-4">Your Devices</h2>
      {devices.map((device) => (
        <div key={device.id} className="flex justify-between items-center p-2 border rounded mb-2">
          <span>{device.name}</span>
          <button
            onClick={() => handleConnect(device.id)}
            className="px-3 py-1 bg-green-600 text-white rounded"
          >
            Connect
          </button>
        </div>
      ))}
    </div>
  );
};
