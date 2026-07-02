import React, { useCallback, useEffect, useState } from 'react';
import { useAuthStore } from '../stores/authStore';
import { deviceService, sessionService, type Device } from '../services/session';
import { RemoteScreen } from './RemoteScreen';
import { RegisterDevice } from './RegisterDevice';
import { ContactsManager } from './ContactsManager';
import { PendingSessions } from './PendingSessions';
import { ScreenShareHost } from './ScreenShareHost';

export const SessionManager: React.FC = () => {
  const token = useAuthStore((state) => state.token);
  const logout = useAuthStore((state) => state.logout);

  const [devices, setDevices] = useState<Device[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [connecting, setConnecting] = useState<string | null>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [sessionType, setSessionType] = useState<'device' | 'user' | null>(null);
  const [isHosting, setIsHosting] = useState(false);
  const [showRegister, setShowRegister] = useState(false);
  const [activeTab, setActiveTab] = useState<'devices' | 'contacts' | 'device-to-device'>('devices');

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

  const handleConnectToDevice = async (deviceId: string) => {
    if (!token) return;
    setConnecting(deviceId);
    setError(null);
    try {
      const response = await sessionService.createSession(deviceId, token);
      setSessionId(response.session_id);
      setSessionType('device');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start session');
    } finally {
      setConnecting(null);
    }
  };

  const handleConnectToUser = async (contactUserId: string) => {
    if (!token) return;
    setConnecting(contactUserId);
    setError(null);
    try {
      const response = await sessionService.createUserSession(contactUserId, token);
      setSessionId(response.session_id);
      setSessionType('user');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start session');
    } finally {
      setConnecting(null);
    }
  };

  const handleConnectDeviceToDevice = async (hostDeviceId: string, clientDeviceId: string) => {
    if (!token) return;
    setConnecting(hostDeviceId);
    setError(null);
    try {
      const response = await deviceService.createDeviceToDeviceSession(hostDeviceId, clientDeviceId, token);
      setSessionId(response.session_id);
      setSessionType('device');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start device-to-device session');
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
    setSessionType(null);
  };

  const handleAcceptSession = (acceptedSessionId: string) => {
    setSessionId(acceptedSessionId);
    setSessionType('user');
    setIsHosting(true); // When accepting, you become the host (screen sharer)
  };

  const handleStopHosting = () => {
    setIsHosting(false);
    setSessionId(null);
    setSessionType(null);
  };

  if (isHosting && sessionId) {
    return (
      <ScreenShareHost
        signalingUrl={`ws://localhost:8080/ws/${sessionId}`}
        onStop={handleStopHosting}
      />
    );
  }

  if (sessionId && !isHosting) {
    return (
      <div className="w-full h-full flex flex-col">
        <div className="flex justify-between items-center p-2 bg-gray-800 text-white">
          <span className="text-sm">
            Connected ({sessionType === 'device' ? 'Device' : 'User Session'})
          </span>
          <button onClick={handleDisconnect} className="px-3 py-1 bg-red-600 rounded text-sm">
            Disconnect
          </button>
        </div>
        <div className="flex-1">
          {sessionType === 'device' ? (
            <RemoteScreen signalingUrl={`ws://localhost:8080/ws/${sessionId}`} />
          ) : (
            <RemoteScreen signalingUrl={`ws://localhost:8080/ws/${sessionId}`} />
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="w-full p-4">
      <PendingSessions onAccept={handleAcceptSession} />

      <div className="flex justify-between items-center mb-4">
        <div className="flex gap-2">
          <button
            onClick={() => setActiveTab('devices')}
            className={`text-sm px-3 py-1 rounded ${
              activeTab === 'devices' ? 'bg-blue-600 text-white' : 'bg-gray-200'
            }`}
          >
            Devices
          </button>
          <button
            onClick={() => setActiveTab('contacts')}
            className={`text-sm px-3 py-1 rounded ${
              activeTab === 'contacts' ? 'bg-blue-600 text-white' : 'bg-gray-200'
            }`}
          >
            Contacts
          </button>
          <button
            onClick={() => setActiveTab('device-to-device')}
            className={`text-sm px-3 py-1 rounded ${
              activeTab === 'device-to-device' ? 'bg-blue-600 text-white' : 'bg-gray-200'
            }`}
          >
            Device-to-Device
          </button>
        </div>
        <button onClick={logout} className="text-sm text-gray-500 hover:text-gray-700">
          Log out
        </button>
      </div>

      {error && (
        <div className="mb-3 p-2 text-sm text-red-700 bg-red-100 rounded">{error}</div>
      )}

      {activeTab === 'devices' ? (
        <div>
          <div className="flex justify-between items-center mb-4">
            <h2 className="text-xl font-bold">Your Devices</h2>
            <button
              onClick={() => setShowRegister(!showRegister)}
              className="text-sm px-3 py-1 bg-blue-600 text-white rounded"
            >
              {showRegister ? 'Cancel' : 'Register Device'}
            </button>
          </div>

          {showRegister && <RegisterDevice />}

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
                    onClick={() => handleConnectToDevice(device.id)}
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
      ) : activeTab === 'contacts' ? (
        <ContactsManager onConnect={handleConnectToUser} />
      ) : (
        <div>
          <h2 className="text-xl font-bold mb-4">Device-to-Device Connection</h2>
          <p className="text-sm text-gray-600 mb-4">
            Connect two of your devices directly. Both devices must have the desktop agent running.
          </p>
          {loading ? (
            <p className="text-gray-500">Loading devices…</p>
          ) : devices.length < 2 ? (
            <div className="text-gray-500">
              <p>You need at least 2 registered devices to connect them.</p>
            </div>
          ) : (
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-2">Host Device (Screen Share)</label>
                <select
                  id="host-device"
                  className="w-full p-2 border rounded"
                  defaultValue=""
                >
                  <option value="">Select host device</option>
                  {devices.map((device) => (
                    <option key={device.id} value={device.id}>
                      {device.name} {device.status === 'online' ? '(Online)' : '(Offline)'}
                    </option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium mb-2">Client Device (Remote View)</label>
                <select
                  id="client-device"
                  className="w-full p-2 border rounded"
                  defaultValue=""
                >
                  <option value="">Select client device</option>
                  {devices.map((device) => (
                    <option key={device.id} value={device.id}>
                      {device.name} {device.status === 'online' ? '(Online)' : '(Offline)'}
                    </option>
                  ))}
                </select>
              </div>
              <button
                onClick={() => {
                  const hostDeviceId = (document.getElementById('host-device') as HTMLSelectElement)?.value;
                  const clientDeviceId = (document.getElementById('client-device') as HTMLSelectElement)?.value;
                  if (hostDeviceId && clientDeviceId && hostDeviceId !== clientDeviceId) {
                    handleConnectDeviceToDevice(hostDeviceId, clientDeviceId);
                  } else {
                    setError('Please select two different devices');
                  }
                }}
                disabled={connecting !== null}
                className="px-4 py-2 bg-blue-600 text-white rounded disabled:opacity-50"
              >
                {connecting ? 'Connecting…' : 'Connect Devices'}
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
