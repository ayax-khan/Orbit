import React, { useCallback, useEffect, useState } from 'react';
import { useAuthStore } from '../stores/authStore';

interface PendingSession {
  id: string;
  session_id: string;
  client_email: string;
  client_name: string;
  started_at: string;
}

interface PendingSessionsProps {
  onAccept: (sessionId: string) => void;
}

export const PendingSessions: React.FC<PendingSessionsProps> = ({ onAccept }) => {
  const token = useAuthStore((state) => state.token);
  const [sessions, setSessions] = useState<PendingSession[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadPendingSessions = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('http://localhost:8080/api/v1/sessions/pending', {
        headers: { Authorization: `Bearer ${token}` },
      });
      if (!response.ok) throw new Error('Failed to load pending sessions');
      const data = await response.json();
      setSessions(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load pending sessions');
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => {
    loadPendingSessions();
    const interval = setInterval(loadPendingSessions, 5000); // Poll every 5 seconds
    return () => clearInterval(interval);
  }, [loadPendingSessions]);

  const handleAccept = async (sessionId: string) => {
    if (!token) return;
    try {
      const response = await fetch(
        `http://localhost:8080/api/v1/sessions/${sessionId}/accept`,
        {
          method: 'POST',
          headers: { Authorization: `Bearer ${token}` },
        }
      );
      if (!response.ok) throw new Error('Failed to accept session');
      onAccept(sessionId);
      await loadPendingSessions();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to accept session');
    }
  };

  if (sessions.length === 0) {
    return null;
  }

  return (
    <div className="w-full p-4 bg-yellow-50 border border-yellow-200 rounded mb-4">
      <h3 className="text-lg font-bold text-yellow-800 mb-2">
        Pending Session Requests
      </h3>
      {error && (
        <div className="mb-2 p-2 text-sm text-red-700 bg-red-100 rounded">{error}</div>
      )}
      {loading ? (
        <p className="text-gray-600">Loading…</p>
      ) : (
        <div className="space-y-2">
          {sessions.map((session) => (
            <div
              key={session.id}
              className="flex justify-between items-center p-2 bg-white border rounded"
            >
              <div>
                <div className="font-medium">{session.client_name}</div>
                <div className="text-sm text-gray-600">{session.client_email}</div>
              </div>
              <button
                onClick={() => handleAccept(session.session_id)}
                className="text-sm px-3 py-1 bg-green-600 text-white rounded"
              >
                Accept
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
