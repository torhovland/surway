use crate::osm::{OsmDocument, OsmNode, OsmWay};

const R: f64 = 6371008.8; // mean Earth radius

#[derive(Clone)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

// Formulas from https://www.movable-type.co.uk/scripts/latlong.html

pub fn distance(c1: &Coord, c2: &Coord) -> f64 {
    // Haversine formula
    let (phi1, phi2) = (c1.phi(), c2.phi());
    let (lambda1, lambda2) = (c1.lambda(), c2.lambda());
    let delta_phi = phi2 - phi1;
    let delta_lambda = lambda2 - lambda1;
    let a = (delta_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
    R * 2.0 * a.sqrt().atan2((1.0 - a).sqrt())
}

pub fn bearing(c1: &Coord, c2: &Coord) -> f64 {
    let (phi1, phi2) = (c1.phi(), c2.phi());
    let (lambda1, lambda2) = (c1.lambda(), c2.lambda());
    let delta_lambda = lambda2 - lambda1;
    let y = delta_lambda.sin() * phi2.cos();
    let x = phi1.cos() * phi2.sin() - phi1.sin() * phi2.cos() * delta_lambda.cos();
    (y.atan2(x).to_degrees() + 360.0) % 360.0
}

pub fn destination(c1: &Coord, bearing: f64, distance: f64) -> Coord {
    let (phi1, lambda1) = (c1.phi(), c1.lambda());
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

fn angular_distance(c1: &Coord, c2: &Coord) -> f64 {
    distance(c1, c2) / R
}

fn delta_theta(c1: &Coord, c2: &Coord, c3: &Coord) -> f64 {
    bearing(c1, c3).to_radians() - bearing(c1, c2).to_radians()
}

fn cross_track_distance(c1: &Coord, c2: &Coord, c3: &Coord) -> f64 {
    R * (angular_distance(c1, c3).sin() * delta_theta(c1, c2, c3).sin()).asin()
}

pub fn along_track_distance(c1: &Coord, c2: &Coord, c3: &Coord) -> f64 {
    // A version with negative sign if we end up before the start point (c1)
    // https://github.com/mrJean1/PyGeodesy/blob/master/pygeodesy/sphericalTrigonometry.py

    let x = (angular_distance(c1, c3).sin() * delta_theta(c1, c2, c3).sin()).asin();

    if x.cos().abs() > f64::EPSILON {
        R * (angular_distance(c1, c3).cos() / x.cos())
            .acos()
            .copysign(delta_theta(c1, c2, c3).cos())
    } else {
        0.0
    }
}

pub fn nearest_point(c1: &Coord, c2: &Coord, c3: &Coord) -> Coord {
    let along_track_distance = along_track_distance(c1, c2, c3);

    if along_track_distance < 0.0 {
        c1.clone()
    } else if along_track_distance > distance(c1, c2) {
        c2.clone()
    } else {
        destination(c1, bearing(c1, c2), along_track_distance)
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

    static BERGEN: Coord = Coord {
        lat: 60.39,
        lon: 5.32,
    };

    static TRONDHEIM: Coord = Coord {
        lat: 63.43,
        lon: 10.39,
    };

    static AALESUND: Coord = Coord {
        lat: 62.47,
        lon: 6.15,
    };

    static STAVANGER: Coord = Coord {
        lat: 58.97,
        lon: 5.73,
    };

    #[test]
    fn test_distance() {
        assert_approx_eq!(distance(&BERGEN, &TRONDHEIM), 429539.2, 0.1);
    }

    #[test]
    fn test_bearing() {
        assert_approx_eq!(bearing(&BERGEN, &TRONDHEIM), 35.93, 0.01);
    }

    #[test]
    fn test_destination() {
        let e = destination(&BERGEN, 35.93, 429539.2);
        assert_approx_eq!(e.lat, TRONDHEIM.lat, 0.001);
        assert_approx_eq!(e.lon, TRONDHEIM.lon, 0.001);
    }

    #[test]
    fn test_destination_north() {
        let s = Coord {
            lat: 53.32,
            lon: -1.72,
        };
        let e = destination(&s, 0.0, 10000.0);
        assert_approx_eq!(e.lat, 53.41, 0.001);
        assert_approx_eq!(e.lon, -1.72, 0.001);
    }

    #[test]
    fn test_cross_track_distance_aalesund() {
        assert_approx_eq!(
            cross_track_distance(&BERGEN, &TRONDHEIM, &AALESUND),
            -101293.1,
            0.1
        );
    }

    #[test]
    fn test_cross_track_distance_stavanger() {
        assert_approx_eq!(
            cross_track_distance(&BERGEN, &TRONDHEIM, &STAVANGER),
            111627.7,
            0.1
        );
    }

    #[test]
    fn test_along_track_distance_aalesund() {
        assert_approx_eq!(
            along_track_distance(&BERGEN, &TRONDHEIM, &AALESUND),
            212561.3,
            0.1
        );
    }

    #[test]
    fn test_along_track_distance_stavanger() {
        assert_approx_eq!(
            along_track_distance(&BERGEN, &TRONDHEIM, &STAVANGER),
            -114024.1,
            0.1
        );
    }

    #[test]
    fn test_nearest_point_aalesund() {
        let destination = destination(&BERGEN, 35.93, 212561.3);
        let nearest_point = nearest_point(&BERGEN, &TRONDHEIM, &AALESUND);

        assert_approx_eq!(nearest_point.lat, destination.lat, 0.001);
        assert_approx_eq!(nearest_point.lon, destination.lon, 0.001);
    }

    #[test]
    fn test_nearest_point_stavanger() {
        let nearest_point = nearest_point(&BERGEN, &TRONDHEIM, &STAVANGER);

        assert_approx_eq!(nearest_point.lat, BERGEN.lat, 0.001);
        assert_approx_eq!(nearest_point.lon, BERGEN.lon, 0.001);
    }
}
