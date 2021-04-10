use leaflet::{LayerGroup, Map};
use seed::Url;
use serde::Serialize;

use crate::{
    geo::Coord,
    osm::{OsmDocument, OsmWay},
};

pub struct Model {
    pub route: Route,
    pub map: Option<Map>,
    pub topology_layer_group: Option<LayerGroup>,
    pub position_layer_group: Option<LayerGroup>,
    pub notes_layer_group: Option<LayerGroup>,
    pub osm: OsmDocument,
    pub position: Coord,
    pub osm_chunk_position: Option<Coord>,
    pub osm_chunk_radius: f64,
    pub osm_chunk_trigger_factor: f64,
    pub notes: Vec<Note>,
    pub new_note: String,
}

#[derive(PartialEq)]
pub enum Route {
    Main,
    EditNote,
    ListNotes,
}

#[derive(Serialize)]
pub struct Note {
    pub datetime: f64,
    pub text: String,
    pub position: Coord,
}

impl Model {
    pub fn find_nearest_way(&self) -> Option<&OsmWay> {
        let nearest_points = self.find_nearest_point_on_each_way();

        let (_, _, way) = nearest_points.iter().min_by(|(_, x, _), (_, y, _)| {
            x.partial_cmp(y).expect("Could not compare distances")
        })?;

        Some(way)
    }

    pub fn is_outside_osm_trigger_box(&self) -> bool {
        if let Some(chunk_pos) = &self.osm_chunk_position {
            let radius = self.osm_chunk_radius * self.osm_chunk_trigger_factor;
            let bbox = chunk_pos.bbox(radius);

            self.position.lat > bbox.upper_right.lat
                || self.position.lon > bbox.upper_right.lon
                || self.position.lat < bbox.lower_left.lat
                || self.position.lon < bbox.lower_left.lon
        } else {
            true
        }
    }

    fn find_nearest_point_on_each_way(&self) -> Vec<(Coord, f64, &OsmWay)> {
        self.osm
            .ways
            .iter()
            .map(|way| way.find_nearest_point(&self.position, &self.osm))
            .collect()
    }
}

impl From<Url> for Route {
    fn from(mut url: Url) -> Self {
        match url.remaining_hash_path_parts().as_slice() {
            ["edit-note"] => Self::EditNote,
            ["list-notes"] => Self::ListNotes,
            _ => Self::Main,
        }
    }
}
