import { useEffect } from 'react';

export const useInput = (dataChannel: RTCDataChannel | null) => {
  useEffect(() => {
    const handleMouseClick = (e: MouseEvent) => {
      if (dataChannel && dataChannel.readyState === 'open') {
        dataChannel.send(JSON.stringify({
          type: 'mouse_click',
          x: e.clientX,
          y: e.clientY
        }));
      }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (dataChannel && dataChannel.readyState === 'open') {
        dataChannel.send(JSON.stringify({
          type: 'key_down',
          key: e.key
        }));
      }
    };

    window.addEventListener('click', handleMouseClick);
    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('click', handleMouseClick);
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [dataChannel]);
};
