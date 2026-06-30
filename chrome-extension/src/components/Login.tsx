import React, { useState } from 'react';
import { useAuthStore } from '../stores/authStore';
import { api } from '../services/api';

export const Login: React.FC = () => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const setToken = useAuthStore((state) => state.setToken);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const response = await api.post<{ access_token: string }>('/auth/login', {
        email,
        password,
        device_fingerprint: 'browser_fingerprint',
      });
      setToken(response.access_token);
    } catch (error) {
      console.error('Login failed:', error);
    }
  };

  return (
    <form onSubmit={handleLogin} className="flex flex-col gap-2">
      <input
        type="email"
        placeholder="Email"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        className="p-2 border rounded"
      />
      <input
        type="password"
        placeholder="Password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        className="p-2 border rounded"
      />
      <button type="submit" className="p-2 bg-blue-600 text-white rounded">
        Login
      </button>
    </form>
  );
};
