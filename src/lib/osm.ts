// OSM API functions: Overpass queries, note posting, OAuth, user info

import type { Coord, BoundingBox, OsmDocument, OsmNode, OsmWay, OsmTag, OAuth2Response, User } from './types';
import { bbox } from './geo';

// ─── Config ───

const OSM_BASE = 'https://www.openstreetmap.org';
const OVERPASS_API = 'https://overpass-api.de/api/interpreter';

function getClientId(): string {
  return import.meta.env.VITE_OSM_CLIENT_ID || '';
}

function getClientSecret(): string {
  return import.meta.env.VITE_OSM_CLIENT_SECRET || '';
}

export function getRedirectUri(): string {
  return import.meta.env.VITE_OSM_REDIRECT_URI || 'http://127.0.0.1:8088/callback';
}

export function getOsmAuthUrl(): string {
  const params = new URLSearchParams({
    client_id: getClientId(),
    redirect_uri: getRedirectUri(),
    response_type: 'code',
    scope: 'write_notes',
  });
  return `${OSM_BASE}/oauth2/authorize?${params.toString()}`;
}

// ─── Fetch OSM data from Overpass API ───

export async function fetchOsmData(
  center: Coord,
  radius: number
): Promise<OsmDocument> {
  const bb = bbox(center, radius);
  // Overpass bbox format: south,west,north,east
  const bboxStr = `${bb.south},${bb.west},${bb.north},${bb.east}`;
  const query = `[out:xml][timeout:25]; way[highway](${bboxStr}); (._;>;); out;`;

  const response = await fetch(OVERPASS_API, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: `data=${encodeURIComponent(query)}`,
  });

  if (!response.ok) {
    if (response.status === 429 || response.status === 504) {
      // Rate limited or gateway timeout — return empty, caller should retry
      return { nodes: new Map(), ways: [] };
    }
    throw new Error(`Overpass API error: ${response.status}`);
  }

  const xmlText = await response.text();
  return parseOsmXml(xmlText);
}

function parseOsmXml(xml: string): OsmDocument {
  const parser = new DOMParser();
  const doc = parser.parseFromString(xml, 'text/xml');
  const nodes = new Map<string, OsmNode>();
  const ways: OsmWay[] = [];

  const nodeElements = doc.querySelectorAll('node');
  nodeElements.forEach((el) => {
    const id = el.getAttribute('id');
    const lat = parseFloat(el.getAttribute('lat') || '0');
    const lon = parseFloat(el.getAttribute('lon') || '0');
    if (id) nodes.set(id, { id, lat, lon });
  });

  const wayElements = doc.querySelectorAll('way');
  wayElements.forEach((el) => {
    const id = el.getAttribute('id') || '';
    const nodeRefs: string[] = [];
    el.querySelectorAll('nd').forEach((nd) => {
      const ref = nd.getAttribute('ref');
      if (ref) nodeRefs.push(ref);
    });
    const tags: OsmTag[] = [];
    el.querySelectorAll('tag').forEach((tag) => {
      const k = tag.getAttribute('k') || '';
      const v = tag.getAttribute('v') || '';
      if (k) tags.push({ k, v });
    });
    if (nodeRefs.length >= 2) {
      ways.push({ id, nodeRefs, tags });
    }
  });

  return { nodes, ways };
}

// ─── Post a note to OSM ───

export async function postOsmNote(
  lat: number,
  lon: number,
  text: string,
  accessToken: string | null
): Promise<boolean> {
  const body = new URLSearchParams({
    lat: lat.toString(),
    lon: lon.toString(),
    text,
  });

  const headers: Record<string, string> = {
    'Content-Type': 'application/x-www-form-urlencoded',
  };

  if (accessToken) {
    headers['Authorization'] = `Bearer ${accessToken}`;
  }

  const response = await fetch(`${OSM_BASE}/api/0.6/notes`, {
    method: 'POST',
    headers,
    body: body.toString(),
  });

  return response.ok;
}

// ─── OAuth2 token exchange ───

export async function exchangeToken(code: string): Promise<OAuth2Response> {
  const body = new URLSearchParams({
    client_id: getClientId(),
    client_secret: getClientSecret(),
    redirect_uri: getRedirectUri(),
    grant_type: 'authorization_code',
    code,
  });

  const response = await fetch(`${OSM_BASE}/oauth2/token`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    body: body.toString(),
  });

  if (!response.ok) {
    throw new Error(`Token exchange failed: ${response.status}`);
  }

  return response.json();
}

// ─── Fetch authenticated user details ───

export async function fetchUserDetails(accessToken: string): Promise<User> {
  const response = await fetch(`${OSM_BASE}/api/0.6/user/details.json`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });

  if (!response.ok) {
    throw new Error(`User fetch failed: ${response.status}`);
  }

  const data = await response.json();
  return {
    name: data.user?.display_name || 'Unknown',
    photo: data.user?.img?.href || null,
  };
}