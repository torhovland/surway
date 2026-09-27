import React, { createContext, useContext, useReducer, useCallback, useEffect, useRef } from 'react';
import type {
  Coord, OsmDocument, NearestWayInfo, Note, User, Route,
} from '../lib/types';
import { OSM_CHUNK_RADIUS, OSM_CHUNK_TRIGGER_FACTOR } from '../lib/types';
import * as geo from '../lib/geo';
import * as osmApi from '../lib/osm';
import { loadNotes, saveNotes } from '../lib/storage';

// ─── State ───

interface AppState {
  route: Route;
  position: Coord | null;
  osmData: OsmDocument;
  nearestWay: NearestWayInfo | null;
  notes: Note[];
  editingNote: Note | null;
  editingText: string;
  trackPosition: boolean;
  wakeLock: boolean;
  wakeLockSentinel: WakeLockSentinel | null;
  accessToken: string | null;
  user: User | null;
  osmChunkPosition: Coord | null;
  osmChunkRadius: number;
  osmChunkTriggerFactor: number;
  downloading: boolean;
  postingNote: boolean;
}

const initialState: AppState = {
  route: 'main',
  position: null,
  osmData: { nodes: new Map(), ways: [] },
  nearestWay: null,
  notes: loadNotes(),
  editingNote: null,
  editingText: '',
  trackPosition: true,
  wakeLock: false,
  wakeLockSentinel: null,
  accessToken: null,
  user: null,
  osmChunkPosition: null,
  osmChunkRadius: OSM_CHUNK_RADIUS,
  osmChunkTriggerFactor: OSM_CHUNK_TRIGGER_FACTOR,
  downloading: false,
  postingNote: false,
};

// ─── Actions ───

type Action =
  | { type: 'SET_ROUTE'; route: Route }
  | { type: 'SET_POSITION'; position: Coord }
  | { type: 'SET_OSM_DATA'; data: OsmDocument; chunkPosition: Coord }
  | { type: 'SET_NEAREST_WAY'; way: NearestWayInfo | null }
  | { type: 'SET_NOTES'; notes: Note[] }
  | { type: 'ADD_NOTE'; note: Note }
  | { type: 'UPDATE_NOTE'; note: Note }
  | { type: 'DELETE_NOTE'; id: number }
  | { type: 'SET_EDITING_NOTE'; note: Note | null; text?: string }
  | { type: 'SET_EDITING_TEXT'; text: string }
  | { type: 'SET_TRACK_POSITION'; on: boolean }
  | { type: 'SET_WAKE_LOCK'; on: boolean; sentinel: WakeLockSentinel | null }
  | { type: 'SET_ACCESS_TOKEN'; token: string | null }
  | { type: 'SET_USER'; user: User | null }
  | { type: 'SET_OSM_CHUNK_POSITION'; pos: Coord | null }
  | { type: 'SET_DOWNLOADING'; downloading: boolean }
  | { type: 'SET_POSTING_NOTE'; posting: boolean }
  | { type: 'MARK_NOTE_UPLOADED'; id: number };

function reducer(state: AppState, action: Action): AppState {
  switch (action.type) {
    case 'SET_ROUTE':
      return { ...state, route: action.route };

    case 'SET_POSITION':
      return { ...state, position: action.position };

    case 'SET_OSM_DATA':
      return { ...state, osmData: action.data, osmChunkPosition: action.chunkPosition };

    case 'SET_NEAREST_WAY':
      return { ...state, nearestWay: action.way };

    case 'SET_NOTES':
      return { ...state, notes: action.notes };

    case 'ADD_NOTE': {
      const notes = [action.note, ...state.notes];
      saveNotes(notes);
      return { ...state, notes };
    }

    case 'UPDATE_NOTE': {
      const notes = state.notes.map((n) => (n.id === action.note.id ? action.note : n));
      saveNotes(notes);
      return { ...state, notes };
    }

    case 'DELETE_NOTE': {
      const notes = state.notes.filter((n) => n.id !== action.id);
      saveNotes(notes);
      return { ...state, notes };
    }

    case 'SET_EDITING_NOTE':
      return {
        ...state,
        editingNote: action.note,
        editingText: action.text ?? action.note?.text ?? '',
      };

    case 'SET_EDITING_TEXT':
      return { ...state, editingText: action.text };

    case 'SET_TRACK_POSITION':
      return { ...state, trackPosition: action.on };

    case 'SET_WAKE_LOCK':
      return {
        ...state,
        wakeLock: action.on,
        wakeLockSentinel: action.sentinel,
      };

    case 'SET_ACCESS_TOKEN':
      return { ...state, accessToken: action.token };

    case 'SET_USER':
      return { ...state, user: action.user };

    case 'SET_OSM_CHUNK_POSITION':
      return { ...state, osmChunkPosition: action.pos };

    case 'SET_DOWNLOADING':
      return { ...state, downloading: action.downloading };

    case 'SET_POSTING_NOTE':
      return { ...state, postingNote: action.posting };

    case 'MARK_NOTE_UPLOADED': {
      const notes = state.notes.map((n) =>
        n.id === action.id ? { ...n, uploaded: true } : n
      );
      saveNotes(notes);
      return { ...state, notes };
    }

    default:
      return state;
  }
}

// ─── Context ───

interface AppContextValue {
  state: AppState;
  dispatch: React.Dispatch<Action>;
  // Convenience actions
  setRoute: (route: Route) => void;
  setPosition: (pos: Coord) => void;
  triggerOsmDownload: () => void;
  recalculateNearestWay: () => void;
  addNote: (note: Note) => void;
  saveEditingNote: () => void;
  deleteNote: (id: number) => void;
  uploadNote: (id: number) => Promise<void>;
  toggleTrackPosition: () => void;
  toggleWakeLock: () => void;
  handleOAuthCallback: (code: string) => Promise<void>;
  locateNote: (note: Note) => void;
  shouldDownloadOsm: (pos: Coord) => boolean;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);
  const downloadingRef = useRef(false);
  const retryTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // ─── Persist notes on change ───

  useEffect(() => {
    saveNotes(state.notes);
  }, [state.notes]);

  // ─── Check URL for OAuth callback on mount ───

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    if (code) {
      handleOAuthCallback(code);
      // Clean URL
      window.history.replaceState({}, '', window.location.pathname);
    }
  }, []);

  // ─── Action wrappers ───

  const setRoute = useCallback((route: Route) => {
    dispatch({ type: 'SET_ROUTE', route });
  }, []);

  const setPosition = useCallback((pos: Coord) => {
    dispatch({ type: 'SET_POSITION', position: pos });
  }, []);

  const shouldDownloadOsm = useCallback((pos: Coord): boolean => {
    if (!state.osmChunkPosition) return true;
    const d = geo.distance(pos, state.osmChunkPosition);
    return d > state.osmChunkRadius * state.osmChunkTriggerFactor;
  }, [state.osmChunkPosition, state.osmChunkRadius, state.osmChunkTriggerFactor]);

  const triggerOsmDownload = useCallback(async () => {
    const pos = state.position;
    if (!pos || downloadingRef.current) return;

    downloadingRef.current = true;
    dispatch({ type: 'SET_DOWNLOADING', downloading: true });

    try {
      const data = await osmApi.fetchOsmData(pos, state.osmChunkRadius);
      if (data.ways.length > 0) {
        dispatch({ type: 'SET_OSM_DATA', data, chunkPosition: pos });
      }
    } catch {
      // On error, retry after 10s if position exists
      retryTimerRef.current = setTimeout(() => {
        downloadingRef.current = false;
        retryTimerRef.current = null;
        triggerOsmDownload();
      }, 10_000);
      return;
    }

    downloadingRef.current = false;
    dispatch({ type: 'SET_DOWNLOADING', downloading: false });
  }, [state.position, state.osmChunkRadius]);

  const recalculateNearestWay = useCallback(() => {
    const { position, osmData } = state;
    if (!position || osmData.ways.length === 0) {
      dispatch({ type: 'SET_NEAREST_WAY', way: null });
      return;
    }

    let bestWay: NearestWayInfo | null = null;
    let bestDist = Infinity;

    for (const way of osmData.ways) {
      if (way.nodeRefs.length < 2) continue;
      const resolved = geo.resolveWay(way, osmData);
      if (!resolved) continue;

      const d = geo.wayDistance(resolved, position);
      if (d < bestDist) {
        bestDist = d;
        bestWay = {
          id: way.id,
          tags: way.tags,
          startDistance: geo.wayStartDistance(resolved, position),
          endDistance: geo.wayEndDistance(resolved, position),
          wayDistance: d,
        };
      }
    }

    dispatch({ type: 'SET_NEAREST_WAY', way: bestWay });
  }, [state.position, state.osmData]);

  const addNote = useCallback((note: Note) => {
    dispatch({ type: 'ADD_NOTE', note });
  }, []);

  const saveEditingNote = useCallback(() => {
    const { editingNote, editingText, position } = state;
    if (!editingText.trim()) return;

    if (editingNote) {
      dispatch({
        type: 'UPDATE_NOTE',
        note: { ...editingNote, text: editingText.trim() },
      });
    } else if (position) {
      const newNote: Note = {
        id: Date.now() % 0xffffffff,
        time: new Date().toISOString(),
        position,
        text: editingText.trim(),
        uploaded: false,
      };
      dispatch({ type: 'ADD_NOTE', note: newNote });
    }

    dispatch({ type: 'SET_EDITING_NOTE', note: null });
    dispatch({ type: 'SET_ROUTE', route: 'notes' });
  }, [state.editingNote, state.editingText, state.position]);

  const deleteNote = useCallback((id: number) => {
    dispatch({ type: 'DELETE_NOTE', id });
  }, []);

  const uploadNote = useCallback(async (id: number) => {
    const note = state.notes.find((n) => n.id === id);
    if (!note) return;

    dispatch({ type: 'SET_POSTING_NOTE', posting: true });
    try {
      const ok = await osmApi.postOsmNote(
        note.position.lat,
        note.position.lon,
        note.text,
        state.accessToken
      );
      if (ok) {
        dispatch({ type: 'MARK_NOTE_UPLOADED', id });
      }
    } catch {
      // Silently fail
    }
    dispatch({ type: 'SET_POSTING_NOTE', posting: false });
  }, [state.notes, state.accessToken]);

  const toggleTrackPosition = useCallback(() => {
    dispatch({ type: 'SET_TRACK_POSITION', on: !state.trackPosition });
  }, [state.trackPosition]);

  const toggleWakeLock = useCallback(async () => {
    if (state.wakeLock) {
      // Release
      if (state.wakeLockSentinel) {
        try {
          await state.wakeLockSentinel.release();
        } catch { /* ignore */ }
      }
      dispatch({ type: 'SET_WAKE_LOCK', on: false, sentinel: null });
    } else {
      // Acquire
      try {
        const sentinel = await navigator.wakeLock.request('screen');
        dispatch({ type: 'SET_WAKE_LOCK', on: true, sentinel });
        sentinel.addEventListener('release', () => {
          dispatch({ type: 'SET_WAKE_LOCK', on: false, sentinel: null });
        });
      } catch {
        // Wake Lock not supported or denied
      }
    }
  }, [state.wakeLock, state.wakeLockSentinel]);

  const handleOAuthCallback = useCallback(async (code: string) => {
    try {
      const tokenResp = await osmApi.exchangeToken(code);
      dispatch({ type: 'SET_ACCESS_TOKEN', token: tokenResp.access_token });
      const user = await osmApi.fetchUserDetails(tokenResp.access_token);
      dispatch({ type: 'SET_USER', user });
    } catch (err) {
      console.error('OAuth callback error:', err);
    }
  }, []);

  const locateNote = useCallback((note: Note) => {
    dispatch({ type: 'SET_TRACK_POSITION', on: false });
    // The map will pan via context — see MapView
  }, []);

  const value: AppContextValue = {
    state,
    dispatch,
    setRoute,
    setPosition,
    triggerOsmDownload,
    recalculateNearestWay,
    addNote,
    saveEditingNote,
    deleteNote,
    uploadNote,
    toggleTrackPosition,
    toggleWakeLock,
    handleOAuthCallback,
    locateNote,
    shouldDownloadOsm,
  };

  return (
    <AppContext.Provider value={value}>
      {children}
    </AppContext.Provider>
  );
}

export function useApp(): AppContextValue {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useApp must be used within AppProvider');
  return ctx;
}

export type { AppState, Action };