import React, { useCallback, useEffect, useState } from 'react';
import { useAuthStore } from '../stores/authStore';
import { deviceService, sessionService, type Device } from '../services/session';
import { RemoteScreen } from './RemoteScreen';

export const SessionManager: React.FC = () => {
  const token = useAuthStore((state) => state.token);
  const logout = useAuthStore((state) => state.logout);

  const [devices, setDevices] = useState<Device[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [connecting, setConnecting] = useState<string | null>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);

  const loadDevices = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      setDevices(await deviceService.listDevices(token));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load devices');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => {
    loadDevices();
  }, [loadDevices]);

  const handleConnect = async (deviceId: string) => {
    if (!token) return;
    setConnecting(deviceId);
    setError(null);
    try {
      const response = await sessionService.createSession(deviceId, token);
      setSessionId(response.session_id);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start session');
    } finally {
      setConnecting(null);
    }
  };

  const handleDisconnect = async () => {
    if (sessionId && token) {
      try {
        await sessionService.endSession(sessionId, token);
      } catch {
        // best-effort cleanup; the session will expire server-side anyway
      }
    }
    setSessionId(null);
  };

  if (sessionId) {
    return (
      <div className="w-full h-full flex flex-col">
        <div className="flex justify-between items-center p-2 bg-gray-800 text-white">
          <span className="text-sm">Connected</span>
          <button onClick={handleDisconnect} className="px-3 py-1 bg-red-600 rounded text-sm">
            Disconnect
          </button>
        </div>
        <div className="flex-1">
          <RemoteScreen signalingUrl={`ws://localhost:8080/ws/${sessionId}`} />
        </div>
      </div>
    );
  }

  return (
    <div className="w-full p-4">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-bold">Your Devices</h2>
        <button onClick={logout} className="text-sm text-gray-500 hover:text-gray-700">
          Log out
        </button>
      </div>

      {error && (
        <div className="mb-3 p-2 text-sm text-red-700 bg-red-100 rounded">{error}</div>
      )}

      {loading ? (
        <p className="text-gray-500">Loading devices…</p>
      ) : devices.length === 0 ? (
        <div className="text-gray-500">
          <p>No devices registered yet.</p>
          <p className="text-sm mt-1">Install and pair the ORBIT desktop agent to see it here.</p>
        </div>
      ) : (
        devices.map((device) => {
          const online = device.status === 'online';
          return (
            <div
              key={device.id}
              className="flex justify-between items-center p-2 border rounded mb-2"
            >
              <span className="flex items-center gap-2">
                <span
                  className={`inline-block w-2 h-2 rounded-full ${
                    online ? 'bg-green-500' : 'bg-gray-400'
                  }`}
                />
                {device.name}
              </span>
              <button
                onClick={() => handleConnect(device.id)}
                disabled={!online || connecting === device.id}
                className="px-3 py-1 bg-green-600 text-white rounded disabled:opacity-50"
              >
                {connecting === device.id ? 'Connecting…' : 'Connect'}
              </button>
            </div>
          );
        })
      )}
    </div>
  );
};
