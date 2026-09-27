import { useEffect, useState } from 'react';
import { useMap } from 'react-leaflet';
import L from 'leaflet';
import { useApp } from '../context/AppContext';

export default function WakeLockControl() {
  const { state, toggleWakeLock } = useApp();
  const map = useMap();
  const [supported, setSupported] = useState(false);

  useEffect(() => {
    setSupported('wakeLock' in navigator);
  }, []);

  useEffect(() => {
    if (!supported) return;

    const Control = L.Control.extend({
      onAdd: function () {
        const container = L.DomUtil.create('div', 'leaflet-bar leaflet-control');
        const link = L.DomUtil.create('a', '', container);
        link.href = '#';
        link.title = 'Keep screen on';
        link.innerHTML =
          '<img src="/icons/brightness.svg" alt="Wake Lock" style="width:20px;height:20px;display:block;margin:4px;" />';

        if (state.wakeLock) {
          link.classList.add('icon-enabled');
        }

        L.DomEvent.on(link, 'click', (e: Event) => {
          L.DomEvent.preventDefault(e);
          L.DomEvent.stopPropagation(e);
          toggleWakeLock();
          link.classList.toggle('icon-enabled');
        });

        return container;
      },
    });

    const control = new Control({ position: 'topleft' });
    control.addTo(map);

    return () => {
      control.remove();
    };
  }, [map, toggleWakeLock, state.wakeLock, supported]);

  return null;
}