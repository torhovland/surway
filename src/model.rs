use leaflet::{LayerGroup, Map};

use crate::{
    geo::{distance, nearest_point, Coord},
    osm::{OsmDocument, OsmWay},
};

pub struct Model {
    pub map: Option<Map>,
    pub topology_layer_group: Option<LayerGroup>,
    pub position_layer_group: Option<LayerGroup>,
    pub osm: OsmDocument,
    pub position: Option<Coord>,
    pub osm_chunk_position: Option<Coord>,
    pub osm_chunk_radius: f64,
    pub osm_chunk_trigger_factor: f64,
}

impl Model {
    pub fn find_nearest_point_on_each_way(&self) -> Vec<(Coord, f64, &OsmWay)> {
        match &self.position {
            None => vec![],
            Some(pos) => self
                .osm
                .ways
                .iter()
                .map(|way| {
                    way.points(&self.osm)
                        .windows(2)
                        .map(|line_segment| {
                            let a = line_segment[0];
                            let b = line_segment[1];
                            let destination = nearest_point(&a.into(), &b.into(), pos);
                            let distance = distance(pos, &destination);
                            (destination, distance, way)
                        })
                        .min_by(|(_, x, _), (_, y, _)| {
                            x.partial_cmp(y).expect("Could not compare distances")
                        })
                        .expect("Could not find a nearest distance")
                })
                .collect(),
        }
    }

    pub fn find_nearest_way(&self) -> Option<&OsmWay> {
        let nearest_points = self.find_nearest_point_on_each_way();

        let (_, _, way) = nearest_points.iter().min_by(|(_, x, _), (_, y, _)| {
            x.partial_cmp(y).expect("Could not compare distances")
        })?;

        Some(way)
    }

    pub fn is_outside_osm_trigger_box(&self) -> bool {
        if let (Some(pos), Some(chunk_pos)) = (&self.position, &self.osm_chunk_position) {
            let radius = self.osm_chunk_radius * self.osm_chunk_trigger_factor;
            let bbox = chunk_pos.bbox(radius);

            pos.lat > bbox.upper_right.lat
                || pos.lon > bbox.upper_right.lon
                || pos.lat < bbox.lower_left.lat
                || pos.lon < bbox.lower_left.lon
        } else {
            true
        }
    }
}
