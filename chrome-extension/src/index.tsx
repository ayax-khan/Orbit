import React from 'react';
import ReactDOM from 'react-dom/client';
import './styles/globals.css';
import { Login } from './components/Login';
import { useAuthStore } from './stores/authStore';

const App = () => {
  const token = useAuthStore((state) => state.token);

  return (
    <div className="flex flex-col items-center justify-center h-full p-4">
      <h1 className="text-2xl font-bold text-blue-600 mb-4">ORBIT Remote</h1>
      {token ? (
        <p>Logged in!</p>
      ) : (
        <Login />
      )}
    </div>
  );
};

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
