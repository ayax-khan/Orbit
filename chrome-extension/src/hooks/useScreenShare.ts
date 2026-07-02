import { useEffect, useRef, useState } from 'react';

export interface ScreenShareConnection {
  /** Media stream being shared (screen + audio) */
  stream: MediaStream | null;
  /** Data channel for input events */
  dataChannel: RTCDataChannel | null;
  /** Human-readable connection state */
  status: string;
  error: string | null;
  /** Function to stop sharing */
  stopSharing: () => void;
}

/**
 * Captures screen and audio using getDisplayMedia and establishes WebRTC
 * connection as the host (screen sharer).
 */
export const useScreenShare = (
  signalingUrl: string,
  isSharing: boolean
): ScreenShareConnection => {
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [dataChannel, setDataChannel] = useState<RTCDataChannel | null>(null);
  const [status, setStatus] = useState('idle');
  const [error, setError] = useState<string | null>(null);
  const pcRef = useRef<RTCPeerConnection | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  const stopSharing = () => {
    if (stream) {
      stream.getTracks().forEach(track => track.stop());
      setStream(null);
    }
    if (pcRef.current) {
      pcRef.current.close();
      pcRef.current = null;
    }
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    setDataChannel(null);
    setStatus('idle');
  };

  useEffect(() => {
    if (!isSharing) {
      stopSharing();
      return;
    }

    setError(null);
    setStatus('starting');

    const startSharing = async () => {
      try {
        // Capture screen with system audio
        const displayStream = await navigator.mediaDevices.getDisplayMedia({
          video: {
            displaySurface: 'monitor',
            frameRate: { ideal: 30 },
            width: { ideal: 1920 },
            height: { ideal: 1080 }
          },
          audio: {
            echoCancellation: true,
            noiseSuppression: true,
            sampleRate: 44100
          }
        });

        setStream(displayStream);
        setStatus('capturing');

        // Create WebRTC peer connection
        const pc = new RTCPeerConnection({
          iceServers: [{ urls: 'stun:stun.l.google.com:19302' }],
        });

        pcRef.current = pc;

        // Add screen video track
        const videoTrack = displayStream.getVideoTracks()[0];
        if (videoTrack) {
          pc.addTrack(videoTrack, displayStream);
        }

        // Add audio track if available
        const audioTrack = displayStream.getAudioTracks()[0];
        if (audioTrack) {
          pc.addTrack(audioTrack, displayStream);
        }

        pc.onconnectionstatechange = () => {
          setStatus(pc.connectionState);
          if (pc.connectionState === 'failed') {
            setError('WebRTC connection failed');
          }
        };

        // Listen for incoming data channel (created by client)
        pc.ondatachannel = (event) => {
          const channel = event.channel;
          channel.binaryType = 'arraybuffer';
          channel.onopen = () => setDataChannel(channel);
          channel.onclose = () => setDataChannel(null);
          channel.onerror = () => setError('Data channel error');
        };

        // Connect to signaling server
        const ws = new WebSocket(signalingUrl);
        wsRef.current = ws;
        ws.onerror = () => setError('Signaling server connection error');

        pc.onicecandidate = (event) => {
          if (event.candidate && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: 'ice', candidate: event.candidate.toJSON() }));
          }
        };

        ws.onmessage = async (event) => {
          try {
            const data = JSON.parse(event.data);
            if (data.type === 'offer') {
              await pc.setRemoteDescription(new RTCSessionDescription(data.sdp));
              const answer = await pc.createAnswer();
              await pc.setLocalDescription(answer);
              ws.send(JSON.stringify({ type: 'answer', sdp: answer }));
            } else if (data.type === 'ice' && data.candidate) {
              await pc.addIceCandidate(new RTCIceCandidate(data.candidate));
            }
          } catch (err) {
            setError('Signaling message error');
          }
        };

        ws.onopen = () => {
          setStatus('connected');
        };

        // Handle user stopping screen share via browser UI
        displayStream.getVideoTracks()[0].onended = () => {
          stopSharing();
        };

      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to capture screen');
        setStatus('error');
      }
    };

    startSharing();

    return () => {
      stopSharing();
    };
  }, [isSharing, signalingUrl]);

  return { stream, dataChannel, status, error, stopSharing };
};
