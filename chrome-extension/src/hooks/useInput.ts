import { useEffect } from 'react';

/**
 * Forwards mouse and keyboard input from a surface element to the host over
 * the WebRTC data channel, using the agent's JSON input protocol.
 *
 * Pointer coordinates are mapped from the on-screen surface into the host's
 * pixel space so clicks land where the user expects regardless of scaling.
 */
export const useInput = (
  dataChannel: RTCDataChannel | null,
  surfaceRef: React.RefObject<HTMLElement | null>,
) => {
  useEffect(() => {
    const surface = surfaceRef.current;
    if (!surface) return;

    const send = (payload: Record<string, unknown>) => {
      if (dataChannel && dataChannel.readyState === 'open') {
        dataChannel.send(JSON.stringify(payload));
      }
    };

    // Map a browser pointer position to host pixel coordinates using the
    // intrinsic size of the rendered media (video/canvas), accounting for
    // object-contain letterboxing.
    const toHostCoords = (e: MouseEvent): { x: number; y: number } | null => {
      const media = surface.querySelector<HTMLVideoElement | HTMLCanvasElement>(
        'video:not(.hidden), canvas:not(.hidden)',
      );
      if (!media) return null;

      const intrinsicW =
        media instanceof HTMLVideoElement ? media.videoWidth : media.width;
      const intrinsicH =
        media instanceof HTMLVideoElement ? media.videoHeight : media.height;
      if (!intrinsicW || !intrinsicH) return null;

      const rect = media.getBoundingClientRect();
      const scale = Math.min(rect.width / intrinsicW, rect.height / intrinsicH);
      const dispW = intrinsicW * scale;
      const dispH = intrinsicH * scale;
      const offsetX = rect.left + (rect.width - dispW) / 2;
      const offsetY = rect.top + (rect.height - dispH) / 2;

      const x = Math.round((e.clientX - offsetX) / scale);
      const y = Math.round((e.clientY - offsetY) / scale);
      if (x < 0 || y < 0 || x > intrinsicW || y > intrinsicH) return null;
      return { x, y };
    };

    const buttonName = (b: number) => (b === 2 ? 'right' : b === 1 ? 'middle' : 'left');

    const handleMouseMove = (e: MouseEvent) => {
      const c = toHostCoords(e);
      if (c) send({ type: 'mouse_move', x: c.x, y: c.y });
    };

    const handleClick = (e: MouseEvent) => {
      const c = toHostCoords(e);
      if (c) send({ type: 'mouse_click', x: c.x, y: c.y, button: buttonName(e.button) });
    };

    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault();
      const c = toHostCoords(e);
      if (c) send({ type: 'mouse_click', x: c.x, y: c.y, button: 'right' });
    };

    const handleWheel = (e: WheelEvent) => {
      e.preventDefault();
      send({
        type: 'mouse_scroll',
        delta_x: Math.sign(e.deltaX),
        delta_y: Math.sign(e.deltaY),
      });
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      send({ type: 'key_down', key: e.key });
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      e.preventDefault();
      send({ type: 'key_up', key: e.key });
    };

    surface.addEventListener('mousemove', handleMouseMove);
    surface.addEventListener('click', handleClick);
    surface.addEventListener('contextmenu', handleContextMenu);
    surface.addEventListener('wheel', handleWheel, { passive: false });
    surface.addEventListener('keydown', handleKeyDown);
    surface.addEventListener('keyup', handleKeyUp);

    return () => {
      surface.removeEventListener('mousemove', handleMouseMove);
      surface.removeEventListener('click', handleClick);
      surface.removeEventListener('contextmenu', handleContextMenu);
      surface.removeEventListener('wheel', handleWheel);
      surface.removeEventListener('keydown', handleKeyDown);
      surface.removeEventListener('keyup', handleKeyUp);
    };
  }, [dataChannel, surfaceRef]);
};
