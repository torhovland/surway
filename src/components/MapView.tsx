import { useEffect, useMemo, useRef, useCallback } from 'react';
import {
  MapContainer,
  TileLayer,
  Polyline,
  Circle,
  Rectangle,
  Marker,
  Popup,
  useMap,
} from 'react-leaflet';
import L from 'leaflet';
import { useApp } from '../context/AppContext';
import type { Coord, ResolvedWay, OsmDocument } from '../lib/types';
import * as geo from '../lib/geo';
import TrackPositionControl from './TrackPositionControl';
import WakeLockControl from './WakeLockControl';

// Fix default marker icon issue with bundlers
import iconUrl from 'leaflet/dist/images/marker-icon.png';
import iconRetinaUrl from 'leaflet/dist/images/marker-icon-2x.png';
import shadowUrl from 'leaflet/dist/images/marker-shadow.png';

// @ts-ignore
delete L.Icon.Default.prototype._getIconUrl;
L.Icon.Default.mergeOptions({ iconUrl, iconRetinaUrl, shadowUrl });

// ─── Inner component that uses the map instance ───

function MapController() {
  const { state } = useApp();
  const map = useMap();
  const prevPosRef = useRef<Coord | null>(null);

  // Pan to position when tracking is on
  useEffect(() => {
    if (!state.position || !state.trackPosition) return;

    if (
      !prevPosRef.current ||
      prevPosRef.current.lat !== state.position.lat ||
      prevPosRef.current.lon !== state.position.lon
    ) {
      map.panTo([state.position.lat, state.position.lon], { animate: true });
      prevPosRef.current = state.position;
    }
  }, [state.position, state.trackPosition, map]);

  return null;
}

// ─── Note markers ───

function NoteMarkers() {
  const { state, deleteNote, uploadNote, setRoute, dispatch, locateNote } = useApp();
  const map = useMap();

  return (
    <>
      {state.notes.map((note) => (
        <Marker
          key={note.id}
          position={[note.position.lat, note.position.lon]}
        >
          <Popup>
            <div className="note-popup">
              <p className="note-popup-text">{note.text}</p>
              <div className="note-popup-time">
                {new Date(note.time).toLocaleString()}
              </div>
              <div className="note-popup-actions">
                <button
                  className="btn btn-sm btn-primary"
                  onClick={() => {
                    dispatch({ type: 'SET_TRACK_POSITION', on: false });
                    map?.panTo([note.position.lat, note.position.lon]);
                  }}
                  title="Locate"
                >
                  <img src="/icons/locate.svg" alt="Locate" className="icon" />
                </button>
                <button
                  className={`btn btn-sm ${note.uploaded ? 'btn-orange' : ''}`}
                  onClick={() => uploadNote(note.id)}
                  title="Upload"
                  disabled={state.postingNote}
                >
                  <img src="/icons/upload.svg" alt="Upload" className="icon" />
                </button>
                <button
                  className="btn btn-sm"
                  onClick={() => {
                    dispatch({ type: 'SET_EDITING_NOTE', note });
                    setRoute('editNote');
                  }}
                  title="Edit"
                >
                  <img src="/icons/pen.svg" alt="Edit" className="icon" />
                </button>
                <button
                  className="btn btn-sm btn-error"
                  onClick={() => deleteNote(note.id)}
                  title="Delete"
                >
                  <img src="/icons/trash.svg" alt="Delete" className="icon" />
                </button>
              </div>
            </div>
          </Popup>
        </Marker>
      ))}
    </>
  );
}

// ─── OSM topology layer ───

function OsmTopology() {
  const { state } = useApp();
  const { osmData, nearestWay, osmChunkPosition, osmChunkRadius, osmChunkTriggerFactor } = state;

  // Build resolved ways and polylines
  const wayLines = useMemo(() => {
    return osmData.ways
      .map((way) => {
        const resolved = geo.resolveWay(way, osmData);
        if (!resolved) return null;
        return {
          id: way.id,
          positions: resolved.nodes.map((n) => [n.lat, n.lon] as [number, number]),
          isNearest: nearestWay?.id === way.id,
        };
      })
      .filter(Boolean) as { id: string; positions: [number, number][]; isNearest: boolean }[];
  }, [osmData, nearestWay]);

  // Bounding boxes
  const chunkBbox = useMemo(() => {
    if (!osmChunkPosition) return null;
    return geo.bbox(osmChunkPosition, osmChunkRadius);
  }, [osmChunkPosition, osmChunkRadius]);

  const triggerBbox = useMemo(() => {
    if (!osmChunkPosition) return null;
    return geo.bbox(osmChunkPosition, osmChunkRadius * osmChunkTriggerFactor);
  }, [osmChunkPosition, osmChunkRadius, osmChunkTriggerFactor]);

  return (
    <>
      {/* Chunk bounding box (red) */}
      {chunkBbox && (
        <Rectangle
          bounds={[
            [chunkBbox.south, chunkBbox.west],
            [chunkBbox.north, chunkBbox.east],
          ]}
          pathOptions={{ color: '#e53935', weight: 2, fillOpacity: 0 }}
        />
      )}

      {/* Trigger bounding box (orange) */}
      {triggerBbox && (
        <Rectangle
          bounds={[
            [triggerBbox.south, triggerBbox.west],
            [triggerBbox.north, triggerBbox.east],
          ]}
          pathOptions={{ color: '#ff9800', weight: 2, fillOpacity: 0 }}
        />
      )}

      {/* Highway ways (green) */}
      {wayLines.map((w) => (
        <Polyline
          key={w.id}
          positions={w.positions}
          pathOptions={
            w.isNearest
              ? { color: '#2196f3', weight: 5 }
              : { color: '#4caf50', weight: 2, fillOpacity: 1 }
          }
        />
      ))}

      {/* User position dot (blue) */}
      {state.position && (
        <Circle
          center={[state.position.lat, state.position.lon]}
          radius={8}
          pathOptions={{ color: '#2196f3', fillColor: '#2196f3', fillOpacity: 1 }}
        />
      )}
    </>
  );
}

// ─── Main MapView ───

const defaultCenter: [number, number] = [60.39, 5.32]; // Bergen area fallback

export default function MapView() {
  const { state } = useApp();

  const center: [number, number] = state.position
    ? [state.position.lat, state.position.lon]
    : defaultCenter;

  return (
    <div id="map">
      <MapContainer
        center={center}
        zoom={19}
        style={{ height: '100%', width: '100%' }}
        zoomControl={true}
      >
        <TileLayer
          attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
          url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
        />

        <MapController />
        <OsmTopology />
        <NoteMarkers />
        <TrackPositionControl />
        <WakeLockControl />
      </MapContainer>
    </div>
  );
}