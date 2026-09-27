// Geographic math based on movable-type.co.uk spherical formulas

import type { Coord, BoundingBox, OsmNode, ResolvedWay } from './types';

const R = 6_371_008.8; // Earth mean radius in meters

// ─── Helpers ───

function toRadians(deg: number): number {
  return (deg * Math.PI) / 180;
}

function toDegrees(rad: number): number {
  return (rad * 180) / Math.PI;
}

export function sameCoord(a: Coord, b: Coord): boolean {
  return a.lat === b.lat && a.lon === b.lon;
}

// ─── Haversine distance (meters) ───

export function distance(c1: Coord, c2: Coord): number {
  const phi1 = toRadians(c1.lat);
  const phi2 = toRadians(c2.lat);
  const deltaPhi = toRadians(c2.lat - c1.lat);
  const deltaLambda = toRadians(c2.lon - c1.lon);

  const a =
    Math.sin(deltaPhi / 2) * Math.sin(deltaPhi / 2) +
    Math.cos(phi1) * Math.cos(phi2) *
    Math.sin(deltaLambda / 2) * Math.sin(deltaLambda / 2);
  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));

  return R * c;
}

// ─── Initial bearing (degrees) ───

export function bearing(c1: Coord, c2: Coord): number {
  const phi1 = toRadians(c1.lat);
  const phi2 = toRadians(c2.lat);
  const deltaLambda = toRadians(c2.lon - c1.lon);

  const y = Math.sin(deltaLambda) * Math.cos(phi2);
  const x =
    Math.cos(phi1) * Math.sin(phi2) -
    Math.sin(phi1) * Math.cos(phi2) * Math.cos(deltaLambda);

  const theta = Math.atan2(y, x);
  return (toDegrees(theta) + 360) % 360;
}

// ─── Destination point given distance and bearing ───

export function destination(c: Coord, brng: number, distMeters: number): Coord {
  const phi1 = toRadians(c.lat);
  const lambda1 = toRadians(c.lon);
  const theta = toRadians(brng);
  const delta = distMeters / R;

  const phi2 = Math.asin(
    Math.sin(phi1) * Math.cos(delta) +
    Math.cos(phi1) * Math.sin(delta) * Math.cos(theta)
  );

  const lambda2 =
    lambda1 +
    Math.atan2(
      Math.sin(theta) * Math.sin(delta) * Math.cos(phi1),
      Math.cos(delta) - Math.sin(phi1) * Math.sin(phi2)
    );

  return {
    lat: toDegrees(phi2),
    lon: ((toDegrees(lambda2) + 540) % 360) - 180,
  };
}

// ─── Angular distance (radians) ───

export function angularDistance(c1: Coord, c2: Coord): number {
  const phi1 = toRadians(c1.lat);
  const phi2 = toRadians(c2.lat);
  const deltaLambda = toRadians(c2.lon - c1.lon);

  return Math.acos(
    Math.sin(phi1) * Math.sin(phi2) +
    Math.cos(phi1) * Math.cos(phi2) * Math.cos(deltaLambda)
  );
}

// ─── Along-track distance (meters, negative means before start) ───

export function alongTrackDistance(
  c1: Coord,
  c2: Coord,
  c3: Coord
): number {
  const delta13 = angularDistance(c1, c3) * R;
  const theta13 = toRadians(bearing(c1, c3));
  const theta12 = toRadians(bearing(c1, c2));

  return delta13 * Math.cos(theta13 - theta12);
}

// ─── Delta theta helper ───

function deltaTheta(c1: Coord, c2: Coord, c3: Coord): number {
  const theta13 = toRadians(bearing(c1, c3));
  const theta12 = toRadians(bearing(c1, c2));
  return theta13 - theta12;
}

// ─── Nearest point on great-circle segment ───

export function nearestPoint(
  c1: Coord,
  c2: Coord,
  c3: Coord
): Coord {
  const d13 = distance(c1, c3);
  const theta13 = toRadians(bearing(c1, c3));
  const theta12 = toRadians(bearing(c1, c2));

  if (d13 === 0) return { ...c1 };

  const dXt = Math.asin(
    Math.sin(toRadians(d13 / R)) * Math.sin(theta13 - theta12)
  ) * R;

  const dAt = Math.acos(
    Math.cos(toRadians(d13 / R)) / Math.cos(Math.abs(dXt) / R)
  ) * R;

  // clamp to segment
  const dist12 = distance(c1, c2);
  const clampedAt = Math.max(0, Math.min(dAt, dist12));

  return destination(c1, bearing(c1, c2), clampedAt);
}

// ─── Bounding box from center + radius ───

export function bbox(center: Coord, radiusMeters: number): BoundingBox {
  const north = destination(center, 0, radiusMeters);
  const east = destination(center, 90, radiusMeters);
  const south = destination(center, 180, radiusMeters);
  const west = destination(center, 270, radiusMeters);

  return {
    south: south.lat,
    west: west.lon,
    north: north.lat,
    east: east.lon,
  };
}

// ─── OSM way helpers ───

export function resolveWay(way: { nodeRefs: string[]; id: string; tags: any[] }, osm: { nodes: Map<string, OsmNode> }): ResolvedWay | null {
  const nodes = way.nodeRefs
    .map((ref) => osm.nodes.get(ref))
    .filter((n): n is OsmNode => n !== undefined);

  if (nodes.length < 2) return null;

  return {
    id: way.id,
    nodes,
    tags: way.tags,
  };
}

export function wayDistance(way: ResolvedWay, point: Coord): number {
  let minDist = Infinity;
  for (let i = 0; i < way.nodes.length - 1; i++) {
    const np = nearestPoint(way.nodes[i], way.nodes[i + 1], point);
    const d = distance(point, np);
    if (d < minDist) minDist = d;
  }
  return minDist;
}

export function wayStartDistance(way: ResolvedWay, point: Coord): number {
  return distance(way.nodes[0], point);
}

export function wayEndDistance(way: ResolvedWay, point: Coord): number {
  return distance(way.nodes[way.nodes.length - 1], point);
}

// ─── Is coord inside bounding box? ───

export function isInsideBbox(c: Coord, bb: BoundingBox): boolean {
  return c.lat <= bb.north && c.lat >= bb.south &&
    c.lon <= bb.east && c.lon >= bb.west;
}