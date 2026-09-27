import { useEffect, useRef } from 'react';
import { useApp } from '../context/AppContext';
import type { Coord } from '../lib/types';

export function useGeolocation() {
  const { setPosition, shouldDownloadOsm, triggerOsmDownload, state } = useApp();
  const lastPosRef = useRef<Coord | null>(null);
  const watchIdRef = useRef<number | null>(null);

  useEffect(() => {
    if (!('geolocation' in navigator)) return;

    const onSuccess = (pos: GeolocationPosition) => {
      const coord: Coord = {
        lat: pos.coords.latitude,
        lon: pos.coords.longitude,
      };

      // Skip if same as last
      if (
        lastPosRef.current &&
        lastPosRef.current.lat === coord.lat &&
        lastPosRef.current.lon === coord.lon
      ) {
        return;
      }

      lastPosRef.current = coord;
      setPosition(coord);
    };

    const onError = (err: GeolocationPositionError) => {
      console.warn('Geolocation error:', err.message);
    };

    watchIdRef.current = navigator.geolocation.watchPosition(onSuccess, onError, {
      enableHighAccuracy: true,
      maximumAge: 0,
      timeout: 30_000,
    });

    return () => {
      if (watchIdRef.current !== null) {
        navigator.geolocation.clearWatch(watchIdRef.current);
      }
    };
  }, [setPosition]);

  // Trigger OSM download when position changes and we're outside the trigger box
  useEffect(() => {
    if (!state.position) return;
    if (shouldDownloadOsm(state.position)) {
      triggerOsmDownload();
    }
  }, [state.position, shouldDownloadOsm, triggerOsmDownload]);
}