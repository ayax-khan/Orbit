import React, { useEffect, useRef } from 'react';
import { useWebRTC } from '../hooks/useWebRTC';
import { useInput } from '../hooks/useInput';

interface RemoteScreenProps {
  signalingUrl: string;
}

export const RemoteScreen: React.FC<RemoteScreenProps> = ({ signalingUrl }) => {
  const { stream, canvasRef, dataChannel, status } = useWebRTC(signalingUrl);
  const videoRef = useRef<HTMLVideoElement>(null);
  const surfaceRef = useRef<HTMLDivElement>(null);

  // Route mouse/keyboard input over the data channel to the host.
  useInput(dataChannel, surfaceRef);

  useEffect(() => {
    if (videoRef.current && stream) {
      videoRef.current.srcObject = stream;
    }
  }, [stream]);

  const connected = status === 'connected' || !!stream || !!dataChannel;

  return (
    <div
      ref={surfaceRef}
      tabIndex={0}
      className="relative w-full h-full bg-black flex items-center justify-center outline-none"
    >
      {/* Hardware/media path */}
      <video
        ref={videoRef}
        autoPlay
        playsInline
        className={`w-full h-full object-contain ${stream ? '' : 'hidden'}`}
      />
      {/* Software/raw-frame path */}
      <canvas ref={canvasRef} className={`w-full h-full object-contain ${stream ? 'hidden' : ''}`} />

      {!connected && (
        <p className="absolute text-white">Connecting to host… ({status})</p>
      )}
    </div>
  );
};
