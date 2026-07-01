import { useEffect, useRef, useState } from 'react';

export const useWebRTC = (signalingUrl: string) => {
  const [stream, setStream] = useState<MediaStream | null>(null);
  const peerConnection = useRef<RTCPeerConnection | null>(null);
  const socket = useRef<WebSocket | null>(null);

  useEffect(() => {
    const pc = new RTCPeerConnection({
      iceServers: [{ urls: 'stun:stun.l.google.com:19302' }],
    });

    pc.ontrack = (event) => {
      setStream(event.streams[0]);
    };

    pc.onicecandidate = (event) => {
      if (event.candidate && socket.current) {
        socket.current.send(JSON.stringify({ type: 'ice', candidate: event.candidate }));
      }
    };

    pc.ondatachannel = (event) => {
      const channel = event.channel;
      channel.onmessage = (e) => {
        console.log("Received raw frame, size:", e.data.length || e.data.size);
      };
    };

    peerConnection.current = pc;

    // Connect to signaling server
    const ws = new WebSocket(signalingUrl);
    socket.current = ws;

    ws.onmessage = async (event) => {
      const data = JSON.parse(event.data);
      if (data.type === 'answer') {
        await pc.setRemoteDescription(new RTCSessionDescription(data.sdp));
      } else if (data.type === 'ice') {
        await pc.addIceCandidate(new RTCIceCandidate(data.candidate));
      }
    };

    // Initiate offer
    pc.createOffer().then(async (offer) => {
      await pc.setLocalDescription(offer);
      ws.send(JSON.stringify({ type: 'offer', sdp: offer }));
    });

    return () => {
      pc.close();
      ws.close();
    };
  }, [signalingUrl]);

  return stream;
};
