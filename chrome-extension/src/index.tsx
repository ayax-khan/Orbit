import React, { useState } from 'react';
import ReactDOM from 'react-dom/client';
import './styles/globals.css';
import { Login } from './components/Login';
import { Register } from './components/Register';
import { SessionManager } from './components/SessionManager';
import { useAuthStore } from './stores/authStore';

const App = () => {
  const token = useAuthStore((state) => state.token);
  const [isLogin, setIsLogin] = useState(true);

  return (
    <div className="flex flex-col items-center justify-center h-full p-4">
      <h1 className="text-2xl font-bold text-blue-600 mb-4">ORBIT Remote</h1>
      {token ? (
        <SessionManager />
      ) : (
        <>
          {isLogin ? <Login /> : <Register onRegister={() => setIsLogin(true)} />}
          <button onClick={() => setIsLogin(!isLogin)} className="mt-2 text-sm text-blue-500">
            {isLogin ? 'Need an account? Register' : 'Have an account? Login'}
          </button>
        </>
      )}
    </div>
  );
};

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
