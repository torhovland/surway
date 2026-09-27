import { useEffect } from 'react';
import { useMap } from 'react-leaflet';
import L from 'leaflet';
import { useApp } from '../context/AppContext';

// Custom Leaflet control for toggling position tracking
export default function TrackPositionControl() {
  const { state, toggleTrackPosition } = useApp();
  const map = useMap();

  useEffect(() => {
    const Control = L.Control.extend({
      onAdd: function () {
        const container = L.DomUtil.create('div', 'leaflet-bar leaflet-control');
        const link = L.DomUtil.create('a', '', container);
        link.href = '#';
        link.title = 'Track position';
        link.innerHTML = '<img src="/icons/locate.svg" alt="Track" style="width:20px;height:20px;display:block;margin:4px;" />';

        if (state.trackPosition) {
          link.classList.add('icon-enabled');
        }

        L.DomEvent.on(link, 'click', (e: Event) => {
          L.DomEvent.preventDefault(e);
          L.DomEvent.stopPropagation(e);
          toggleTrackPosition();
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
  }, [map, toggleTrackPosition, state.trackPosition]);

  return null;
}