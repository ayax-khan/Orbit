import { useEffect, useRef, useState } from 'react';

/** Header for a raw ORBIT frame: magic(4) + width(4) + height(4) + seq(8). */
const RAW_FRAME_HEADER_BYTES = 20;
const RAW_FRAME_MAGIC = 0x314252_4f; // "ORB1" little-endian as u32 is decoded below

export interface WebRTCConnection {
  /** Media stream, when the host negotiates a media track. */
  stream: MediaStream | null;
  /** Canvas element rendering raw data-channel frames (software path). */
  canvasRef: React.RefObject<HTMLCanvasElement>;
  /** Data channel used to send input events back to the host. */
  dataChannel: RTCDataChannel | null;
  /** Human-readable connection state. */
  status: string;
  error: string | null;
}

/**
 * Establishes the WebRTC connection to the host via the backend signaling
 * relay.
 */
export const useWebRTC = (signalingUrl: string): WebRTCConnection => {
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [dataChannel, setDataChannel] = useState<RTCDataChannel | null>(null);
  const [status, setStatus] = useState('connecting');
  const [error, setError] = useState<string | null>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    setError(null);
    const pc = new RTCPeerConnection({
      iceServers: [{ urls: 'stun:stun.l.google.com:19302' }],
    });

    pc.ontrack = (event) => setStream(event.streams[0]);

    pc.onconnectionstatechange = () => {
      setStatus(pc.connectionState);
      if (pc.connectionState === 'failed') {
        setError('WebRTC connection failed');
      }
    };

    const ws = new WebSocket(signalingUrl);
    ws.onerror = () => setError('Signaling server connection error');

    pc.onicecandidate = (event) => {
      if (event.candidate && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'ice', candidate: event.candidate.toJSON() }));
      }
    };

    // Client creates the data channel: input events out, raw frames in.
    const channel = pc.createDataChannel('orbit', { ordered: true });
    channel.binaryType = 'arraybuffer';
    channel.onopen = () => setDataChannel(channel);
    channel.onclose = () => setDataChannel(null);
    channel.onerror = () => setError('Data channel error');
    channel.onmessage = (e) => renderRawFrame(canvasRef.current, e.data);

    ws.onmessage = async (event) => {
      try {
        const data = JSON.parse(event.data);
        if (data.type === 'answer') {
          await pc.setRemoteDescription(new RTCSessionDescription(data.sdp));
        } else if (data.type === 'ice' && data.candidate) {
          await pc.addIceCandidate(new RTCIceCandidate(data.candidate));
        }
      } catch (err) {
        setError('Signaling message error');
      }
    };

    ws.onopen = async () => {
      const offer = await pc.createOffer({ offerToReceiveVideo: true });
      await pc.setLocalDescription(offer);
      ws.send(JSON.stringify({ type: 'offer', sdp: offer }));
    };

    return () => {
      channel.close();
      pc.close();
      ws.close();
    };
  }, [signalingUrl]);

  return { stream, canvasRef, dataChannel, status, error };
};

/** Decode and paint a raw BGRA frame produced by the software encoder. */
function renderRawFrame(canvas: HTMLCanvasElement | null, data: unknown): void {
  if (!canvas || !(data instanceof ArrayBuffer) || data.byteLength <= RAW_FRAME_HEADER_BYTES) {
    return;
  }
  const view = new DataView(data);
  if (view.getUint32(0, true) !== RAW_FRAME_MAGIC) {
    return;
  }
  const width = view.getUint32(4, true);
  const height = view.getUint32(8, true);
  const pixels = new Uint8ClampedArray(data, RAW_FRAME_HEADER_BYTES);
  if (pixels.length < width * height * 4) {
    return;
  }

  // Convert BGRA -> RGBA in place for the browser's ImageData.
  const rgba = new Uint8ClampedArray(width * height * 4);
  for (let i = 0; i < rgba.length; i += 4) {
    rgba[i] = pixels[i + 2];
    rgba[i + 1] = pixels[i + 1];
    rgba[i + 2] = pixels[i];
    rgba[i + 3] = 255;
  }

  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext('2d');
  ctx?.putImageData(new ImageData(rgba, width, height), 0, 0);
}
