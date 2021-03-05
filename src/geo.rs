use crate::osm::{OsmDocument, OsmNode, OsmWay};

const R: f64 = 6371009.0;

#[derive(Clone)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

// Formulas from https://www.movable-type.co.uk/scripts/latlong.html

pub fn distance(c1: &Coord, c2: &Coord) -> f64 {
    // Haversine formula
    let phi1 = c1.phi();
    let phi2 = c2.phi();
    let lambda1 = c1.lambda();
    let lambda2 = c2.lambda();
    let delta_phi = phi2 - phi1;
    let delta_lambda = lambda2 - lambda1;
    let a = (delta_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    R * c
}

pub fn bearing(c1: &Coord, c2: &Coord) -> f64 {
    let phi1 = c1.phi();
    let phi2 = c2.phi();
    let lambda1 = c1.lambda();
    let lambda2 = c2.lambda();
    let delta_lambda = lambda2 - lambda1;
    let y = delta_lambda.sin() * phi2.cos();
    let x = phi1.cos() * phi2.sin() - phi1.sin() * phi2.cos() * delta_lambda.cos();
    let theta = y.atan2(x);
    (theta.to_degrees() + 360.0) % 360.0
}

pub fn destination(c1: &Coord, bearing: f64, distance: f64) -> Coord {
    let phi1 = c1.phi();
    let lambda1 = c1.lambda();
    let brng = bearing.to_radians();
    let phi2 =
        (phi1.sin() * (distance / R).cos() + phi1.cos() * (distance / R).sin() * brng.cos()).asin();
    let lambda2 = lambda1
        + (brng.sin() * (distance / R).sin() * phi1.cos())
            .atan2((distance / R).cos() - phi1.sin() * phi2.sin());

    Coord {
        lat: phi2.to_degrees(),
        lon: ((lambda2 + 540.0) % 360.0 - 180.0).to_degrees(),
    }
}

pub fn cross_track_distance(c1: &Coord, c2: &Coord, c3: &Coord) -> f64 {
    let d13 = distance(c1, c3);
    let d_13_angular = d13 / R;
    let theta13 = bearing(c1, c3).to_radians();
    let theta12 = bearing(c1, c2).to_radians();
    let delta_theta = theta13 - theta12;
    let x = (d_13_angular.sin() * delta_theta.sin()).asin();
    R * x
}

pub fn along_track_distance(c1: &Coord, c2: &Coord, c3: &Coord) -> f64 {
    // A version with negative sign if we end up before the start point (c1)
    // https://github.com/mrJean1/PyGeodesy/blob/master/pygeodesy/sphericalTrigonometry.py

    let d13 = distance(c1, c3);
    let d_13_angular = d13 / R;
    let theta13 = bearing(c1, c3).to_radians();
    let theta12 = bearing(c1, c2).to_radians();
    let delta_theta = theta13 - theta12;
    let x = (d_13_angular.sin() * delta_theta.sin()).asin();

    if x.cos().abs() > f64::EPSILON {
        R * (d_13_angular.cos() / x.cos())
            .acos()
            .copysign(delta_theta.cos())
    } else {
        0.0
    }
}

pub fn nearest_point(c1: Coord, c2: Coord, c3: Coord) -> Coord {
    let length = distance(&c1, &c2);
    let along_track_distance = along_track_distance(&c1, &c2, &c3);
    let bearing = bearing(&c1, &c2);

    if along_track_distance < 0.0 {
        c1
    } else if along_track_distance > length {
        c2
    } else {
        destination(&c1, bearing, along_track_distance)
    }
}

impl Coord {
    fn phi(self: &Coord) -> f64 {
        self.lat.to_radians()
    }

    fn lambda(self: &Coord) -> f64 {
        self.lon.to_radians()
    }
}

impl OsmNode {
    fn distance(self: &OsmNode, coord: &Coord) -> f64 {
        distance(&self.into(), coord)
    }
}

impl OsmWay {
    pub fn distance(self: &OsmWay, coord: &Coord, osm: &OsmDocument) -> f64 {
        self.points(osm)
            .iter()
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

    const BERGEN: Coord = Coord {
        lat: 60.390321,
        lon: 5.328394,
    };

    const TRONDHEIM: Coord = Coord {
        lat: 63.387661,
        lon: 10.434604,
    };

    const FORDE: Coord = Coord {
        lat: 61.452202,
        lon: 5.857147,
    };

    #[test]
    fn test_distance_points() {
        let bergen = BERGEN;
        let trondheim = TRONDHEIM;
        assert_approx_eq!(distance(&bergen, &trondheim), 427117.53826249886, 0.1);
    }

    #[test]
    fn test_bearing() {
        let bergen = BERGEN;
        let trondheim = TRONDHEIM;
        assert_approx_eq!(bearing(&bergen, &trondheim), 36.52253184347995, 0.1);
    }

    #[test]
    fn test_destination() {
        let bergen = BERGEN;
        let trondheim = TRONDHEIM;
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
        let bergen = BERGEN;
        let trondheim = TRONDHEIM;
        let forde = FORDE;
        assert_approx_eq!(
            cross_track_distance(&bergen, &trondheim, &forde),
            -47755.6,
            0.1
        );
    }

    #[test]
    fn test_along_track_distance() {
        let bergen = BERGEN;
        let trondheim = TRONDHEIM;
        let forde = FORDE;
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
        assert_approx_eq!(cross_track_distance(&s, &e, &p), -307.6, 0.1);
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
        assert_approx_eq!(along_track_distance(&s, &e, &p2), 7587.6, 0.1);
        assert_approx_eq!(along_track_distance(&s, &e, &p1), -7702.7, 0.1);
    }
}
