use crate::osm::{OsmDocument, OsmNode, OsmWay};

const R: f64 = 6371009.0;

#[derive(Clone)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

pub fn bearing(a: &Coord, b: &Coord) -> f64 {
    let pa = a.lat.to_radians();
    let pb = b.lat.to_radians();
    let la = a.lon.to_radians();
    let lb = b.lon.to_radians();
    let y = (lb - la).sin() * pb.cos();
    let x = pa.cos() * pb.sin() - pa.sin() * pb.cos() * (lb - la).cos();
    let t = y.atan2(x);
    (t.to_degrees() + 360.0) % 360.0
}

pub fn distance_points(a: &Coord, b: &Coord) -> f64 {
    let ta = a.lat.to_radians();
    let tb = b.lat.to_radians();
    let dt = (a.lat - b.lat).to_radians();
    let dl = (a.lon - b.lon).to_radians();
    let a = (dt / 2.0).sin().powi(2) + ta.cos() * tb.cos() * (dl / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    R * c
}

fn distance_line_point(line_a: &Coord, line_b: &Coord, c: &Coord) -> f64 {
    let d_ac = distance_points(line_a, c);
    let angular_d_ac = d_ac / R;
    let t_ac = bearing(line_a, c).to_radians();
    let t_ab = bearing(line_a, line_b).to_radians();
    R * (angular_d_ac.sin() * (t_ac - t_ab).sin()).asin()
}

fn along_track_distance(line_a: &Coord, line_b: &Coord, c: &Coord) -> f64 {
    let d_ac = distance_points(line_a, c);
    let angular_d_ac = d_ac / R;
    R * (angular_d_ac.cos() / (distance_line_point(line_a, line_b, c) / R).cos()).acos()
}

pub fn along_track_distance2(line_a: &Coord, line_b: &Coord, c: &Coord) -> f64 {
    let d_ac = distance_points(line_a, c);
    let angular_d_ac = d_ac / R;
    let b = bearing(line_a, c).to_radians();
    let e = bearing(line_a, line_b).to_radians();
    let x = (angular_d_ac.sin() * (b - e).sin()).asin();
    let b2 = e - b;
    let cx = x.cos();

    if cx.abs() > f64::EPSILON {
        R * (angular_d_ac.cos() / cx).acos().copysign(b2.cos())
    } else {
        0.0
    }
}

pub fn destination(start: &Coord, bearing: f64, d: f64) -> Coord {
    let p1 = start.lat.to_radians();
    let l1 = start.lon.to_radians();
    let brng = bearing.to_radians();
    let p2 = (p1.sin() * (d / R).cos() + p1.cos() * (d / R).sin() * brng.cos()).asin();
    let l2 =
        l1 + (brng.sin() * (d / R).sin() * p1.cos()).atan2((d / R).cos() - p1.sin() * p2.sin());
    Coord {
        lat: p2.to_degrees(),
        lon: l2.to_degrees(),
        //lon: ((l2 + 540.0) % 360.0 - 180.0).to_degrees(),
    }
}

impl OsmNode {
    fn distance(self: &OsmNode, coord: &Coord) -> f64 {
        distance_points(&self.into(), coord)
    }
}

impl OsmWay {
    pub fn distance(self: &OsmWay, coord: &Coord, osm: &OsmDocument) -> f64 {
        self.points(osm)
            .map(|point| point.distance(coord))
            .min_by(|a, b| a.partial_cmp(b).expect("Tried to compare a NaN"))
            .unwrap_or(f64::INFINITY) // In case of a way with no points
    }
}

impl From<&OsmNode> for Coord {
    fn from(node: &OsmNode) -> Self {
        Coord {
            lat: node.lat,
            lon: node.lon,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_approx_eq::assert_approx_eq;

    const Bergen: Coord = Coord {
        lat: 60.390321,
        lon: 5.328394,
    };

    const Trondheim: Coord = Coord {
        lat: 63.387661,
        lon: 10.434604,
    };

    const Forde: Coord = Coord {
        lat: 61.452202,
        lon: 5.857147,
    };

    #[test]
    fn test_distance_points() {
        let bergen = Bergen;
        let trondheim = Trondheim;
        assert_approx_eq!(
            distance_points(&bergen, &trondheim),
            427117.53826249886,
            0.1
        );
    }

    #[test]
    fn test_bearing() {
        let bergen = Bergen;
        let trondheim = Trondheim;
        assert_approx_eq!(bearing(&bergen, &trondheim), 36.52253184347995, 0.1);
    }

    #[test]
    fn test_destination() {
        let bergen = Bergen;
        let trondheim = Trondheim;
        let e = destination(&bergen, 36.52253184347995, 427117.53826249886);
        assert_approx_eq!(e.lat, trondheim.lat, 0.0000001);
        assert_approx_eq!(e.lon, trondheim.lon, 0.0000001);
    }

    #[test]
    fn test_destination_north() {
        let s = Coord {
            lat: 53.3206,
            lon: -1.7297,
        };
        let e = destination(&s, 0.0, 10000.0);
        assert_approx_eq!(e.lat, 53.4, 0.1);
        assert_approx_eq!(e.lon, -1.7297, 0.1);
    }

    #[test]
    fn test_distance_line_point() {
        let bergen = Bergen;
        let trondheim = Trondheim;
        let forde = Forde;
        assert_approx_eq!(
            distance_line_point(&bergen, &trondheim, &forde),
            -47755.6,
            0.1
        );
    }

    #[test]
    fn test_along_track_distance() {
        let bergen = Bergen;
        let trondheim = Trondheim;
        let forde = Forde;
        assert_approx_eq!(
            along_track_distance(&bergen, &trondheim, &forde),
            111704.2,
            0.1
        );
    }

    #[test]
    fn test_distance_line_point_pygeodesy() {
        // https://github.com/mrJean1/PyGeodesy/blob/711eac98ded1391a0099edfa5f8882306031b7e0/pygeodesy/sphericalTrigonometry.py
        let s = Coord {
            lat: 53.3206,
            lon: -1.7297,
        };
        let e = Coord {
            lat: 53.1887,
            lon: 0.1334,
        };
        let p = Coord {
            lat: 53.2611,
            lon: -0.7972,
        };
        assert_approx_eq!(distance_line_point(&s, &e, &p), -307.6, 0.1);
    }

    #[test]
    fn test_along_track_distance_pygeodesy() {
        // https://github.com/mrJean1/PyGeodesy/issues/7
        let s = Coord {
            lat: 53.3206,
            lon: -1.7297,
        };
        let e = Coord {
            lat: 53.1887,
            lon: 0.1334,
        };
        let p1 = Coord {
            lat: 53.36366,
            lon: -1.83883,
        };
        let p2 = Coord {
            lat: 53.35423,
            lon: -1.60881,
        };
        assert_approx_eq!(along_track_distance2(&s, &e, &p2), 7587.6, 0.1);
        assert_approx_eq!(along_track_distance2(&s, &e, &p1), -7702.7, 0.1);
    }
}
