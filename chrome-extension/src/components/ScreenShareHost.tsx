import React from 'react';
import { useScreenShare } from '../hooks/useScreenShare';
import { useInput } from '../hooks/useInput';

interface ScreenShareHostProps {
  signalingUrl: string;
  onStop: () => void;
}

export const ScreenShareHost: React.FC<ScreenShareHostProps> = ({ signalingUrl, onStop }) => {
  const { stream, dataChannel, status, error, stopSharing } = useScreenShare(signalingUrl, true);
  const videoRef = React.useRef<HTMLVideoElement>(null);
  const surfaceRef = React.useRef<HTMLDivElement>(null);

  // Route mouse/keyboard input over the data channel to the client.
  useInput(dataChannel, surfaceRef);

  React.useEffect(() => {
    if (videoRef.current && stream) {
      videoRef.current.srcObject = stream;
    }
  }, [stream]);

  const handleStop = () => {
    stopSharing();
    onStop();
  };

  const connected = status === 'connected' || !!stream || !!dataChannel;

  return (
    <div className="w-full h-full flex flex-col">
      <div className="flex justify-between items-center p-2 bg-gray-800 text-white">
        <span className="text-sm">
          Sharing Screen ({status}) {error && `- ${error}`}
        </span>
        <button onClick={handleStop} className="px-3 py-1 bg-red-600 rounded text-sm">
          Stop Sharing
        </button>
      </div>
      <div
        ref={surfaceRef}
        tabIndex={0}
        className="relative w-full h-full bg-black flex items-center justify-center outline-none"
      >
        <video
          ref={videoRef}
          autoPlay
          muted
          playsInline
          className={`w-full h-full object-contain ${stream ? '' : 'hidden'}`}
        />
        {!connected && (
          <p className="absolute text-white">
            {status === 'capturing' ? 'Waiting for connection...' : 'Starting screen share...'}
          </p>
        )}
      </div>
    </div>
  );
};
