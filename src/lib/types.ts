// ─── Geographic types ───

export interface Coord {
  lat: number;
  lon: number;
}

export interface BoundingBox {
  south: number;
  west: number;
  north: number;
  east: number;
}

// ─── OSM data types ───

export interface OsmNode {
  id: string;
  lat: number;
  lon: number;
}

export interface OsmWay {
  id: string;
  nodeRefs: string[];
  tags: OsmTag[];
}

export interface OsmTag {
  k: string;
  v: string;
}

export interface OsmDocument {
  nodes: Map<string, OsmNode>;
  ways: OsmWay[];
}

export interface ResolvedWay {
  id: string;
  nodes: OsmNode[];
  tags: OsmTag[];
}

// ─── Nearest way info ───

export interface NearestWayInfo {
  id: string;
  tags: OsmTag[];
  startDistance: number;
  endDistance: number;
  wayDistance: number;
}

// ─── Notes ───

export interface Note {
  id: number;
  time: string;
  position: Coord;
  text: string;
  uploaded: boolean;
}

// ─── OAuth ───

export interface OAuth2Response {
  access_token: string;
}

export interface User {
  name: string;
  photo: string | null;
}

// ─── App routes ───

export type Route = 'main' | 'notes' | 'editNote' | 'newNote' | 'callback';

// ─── OSM chunk trigger ───

export const OSM_CHUNK_RADIUS = 500; // meters
export const OSM_CHUNK_TRIGGER_FACTOR = 0.8;