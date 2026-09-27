/// <reference types="vite/client" />

interface WakeLockSentinel {
  released: boolean;
  type: string;
  release(): Promise<void>;
  addEventListener(type: 'release', listener: () => void): void;
}

interface Navigator {
  wakeLock: {
    request(type: 'screen'): Promise<WakeLockSentinel>;
  };
}