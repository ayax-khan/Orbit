import { useEffect, useRef, useState } from 'react';

export const useWebRTC = (signalingUrl: string) => {
  const [stream, setStream] = useState<MediaStream | null>(null);
  const peerConnection = useRef<RTCPeerConnection | null>(null);

  useEffect(() => {
    const pc = new RTCPeerConnection({
      iceServers: [{ urls: 'stun:stun.l.google.com:19302' }],
    });

    pc.ontrack = (event) => {
      setStream(event.streams[0]);
    };

    peerConnection.current = pc;

    // TODO: Implement signaling logic here to exchange SDP/ICE candidates
    // with the signaling server to establish connection with Agent.

    return () => {
      pc.close();
    };
  }, [signalingUrl]);

  return stream;
};
