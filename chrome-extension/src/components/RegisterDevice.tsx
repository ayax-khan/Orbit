import React, { useState } from 'react';
import { deviceService } from '../services/session';
import { useAuthStore } from '../stores/authStore';

export const RegisterDevice: React.FC = () => {
  const token = useAuthStore((state) => state.token);
  const [deviceName, setDeviceName] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<{ device_id: string; device_token: string } | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!token || !deviceName.trim()) return;

    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      const result = await deviceService.registerDevice(deviceName.trim(), token);
      setSuccess(result);
      setDeviceName('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to register device');
    } finally {
      setLoading(false);
    }
  };

  if (success) {
    return (
      <div className="p-4 bg-green-50 border border-green-200 rounded">
        <h3 className="font-bold text-green-800 mb-2">Device Registered Successfully!</h3>
        <div className="space-y-2 text-sm">
          <div>
            <span className="font-medium">Device ID:</span>
            <code className="ml-2 bg-white px-2 py-1 rounded">{success.device_id}</code>
          </div>
          <div>
            <span className="font-medium">Device Token:</span>
            <code className="ml-2 bg-white px-2 py-1 rounded text-xs break-all">{success.device_token}</code>
          </div>
          <p className="text-green-700 mt-2">
            Use this token in your desktop agent's ORBIT_DEVICE_TOKEN environment variable.
          </p>
        </div>
        <button
          onClick={() => setSuccess(null)}
          className="mt-3 px-3 py-1 bg-green-600 text-white rounded text-sm"
        >
          Register Another Device
        </button>
      </div>
    );
  }

  return (
    <div className="p-4 bg-gray-50 border rounded">
      <h3 className="font-bold mb-3">Register New Device</h3>
      {error && (
        <div className="mb-3 p-2 text-sm text-red-700 bg-red-100 rounded">{error}</div>
      )}
      <form onSubmit={handleSubmit} className="space-y-3">
        <div>
          <label htmlFor="deviceName" className="block text-sm font-medium mb-1">
            Device Name
          </label>
          <input
            id="deviceName"
            type="text"
            value={deviceName}
            onChange={(e) => setDeviceName(e.target.value)}
            placeholder="e.g., My Desktop PC"
            className="w-full px-3 py-2 border rounded"
            disabled={loading}
          />
        </div>
        <button
          type="submit"
          disabled={loading || !deviceName.trim()}
          className="w-full px-3 py-2 bg-blue-600 text-white rounded disabled:opacity-50"
        >
          {loading ? 'Registering...' : 'Register Device'}
        </button>
      </form>
    </div>
  );
};
