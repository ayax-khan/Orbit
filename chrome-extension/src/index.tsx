import React from 'react';
import ReactDOM from 'react-dom/client';
import './styles/globals.css';

const App = () => {
  return (
    <div className="flex flex-col items-center justify-center h-full p-4">
      <h1 className="text-2xl font-bold text-blue-600">ORBIT Remote</h1>
      <p className="mt-2 text-gray-600">Please sign in to continue.</p>
    </div>
  );
};

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
