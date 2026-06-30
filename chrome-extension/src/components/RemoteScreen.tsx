import React, { useEffect, useRef } from 'react';
import { useWebRTC } from '../hooks/useWebRTC';

interface RemoteScreenProps {
  signalingUrl: string;
}

export const RemoteScreen: React.FC<RemoteScreenProps> = ({ signalingUrl }) => {
  const stream = useWebRTC(signalingUrl);
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (videoRef.current && stream) {
      videoRef.current.srcObject = stream;
    }
  }, [stream]);

  return (
    <div className="w-full h-full bg-black flex items-center justify-center">
      {stream ? (
        <video
          ref={videoRef}
          autoPlay
          playsInline
          className="w-full h-full object-contain"
        />
      ) : (
        <p className="text-white">Connecting to host...</p>
      )}
    </div>
  );
};
