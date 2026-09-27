import { useEffect, useRef, useCallback } from 'react';
import { useApp } from '../context/AppContext';
import type { Coord } from '../lib/types';
import { destination, distance } from '../lib/geo';

// Check if URL has ?random_walk parameter
function isRandomWalkEnabled(): boolean {
  return new URLSearchParams(window.location.search).has('random_walk');
}

// Initial random position
function randomPosition(): Coord {
  return {
    lat: 60.35 + Math.random() * 0.1,
    lon: 5.3 + Math.random() * 0.1,
  };
}

export function useRandomWalk() {
  const { setPosition, state } = useApp();
  const headingRef = useRef(Math.random() * 360);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const step = useCallback(() => {
    // Slightly change heading each step
    headingRef.current += (Math.random() - 0.5) * 30;
    const pos = state.position || randomPosition();
    const newPos = destination(pos, headingRef.current, 5 + Math.random() * 10);
    setPosition(newPos);
  }, [state.position, setPosition]);

  useEffect(() => {
    if (!isRandomWalkEnabled()) return;

    // Set initial random position
    const initPos = randomPosition();
    setPosition(initPos);
    headingRef.current = Math.random() * 360;

    timerRef.current = setInterval(step, 2000);

    return () => {
      if (timerRef.current) {
        clearInterval(timerRef.current);
      }
    };
  }, [setPosition, step]);
}