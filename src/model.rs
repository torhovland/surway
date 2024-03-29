use leaflet::{LayerGroup, Map};
use seed::Url;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt};
use web_sys::WakeLockSentinel;

use crate::{
    geo::Coord,
    js_sys::Date,
    osm::{OsmDocument, OsmWay},
};

pub struct Model {
    pub route: Route,
    pub access_token: Option<String>,
    pub user: Option<User>,
    pub map: Option<Map>,
    pub topology_layer_group: Option<LayerGroup>,
    pub position_layer_group: Option<LayerGroup>,
    pub notes_layer_group: Option<LayerGroup>,
    pub osm: OsmDocument,
    pub position: Coord,
    pub nearest_way_id: Option<String>,
    pub start_distance: Option<f64>,
    pub end_distance: Option<f64>,
    pub way_distance: Option<f64>,
    pub track_position: bool,
    pub osm_chunk_position: Option<Coord>,
    pub osm_chunk_radius: f64,
    pub osm_chunk_trigger_factor: f64,
    pub notes: VecDeque<Note>,
    pub new_note: String,
    pub note_id: Option<NoteId>,
    pub wake_lock_sentinel: Option<WakeLockSentinel>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Main,
    Callback(String),
    EditNote,
    NewNote,
    Notes,
}

#[derive(Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct NoteId(u32);

impl NoteId {
    pub fn new() -> NoteId {
        NoteId(Date::now().round().rem_euclid(2f64.powi(32)) as u32)
    }
}

impl fmt::Display for NoteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Note {
    pub id: NoteId,
    pub time: f64,
    pub position: Coord,
    pub text: String,
    pub uploaded: bool,
}

#[derive(Debug, Deserialize)]
pub struct OAuth2Response {
    pub access_token: String,
}

#[derive(Debug)]
pub struct User {
    pub name: String,
    pub photo: String,
}

#[derive(Debug, Deserialize)]
pub struct UserResponse {
    pub user: UserResponseUser,
}

#[derive(Debug, Deserialize)]
pub struct UserResponseUser {
    pub display_name: String,
    pub img: UserResponseImg,
}

#[derive(Debug, Deserialize)]
pub struct UserResponseImg {
    pub href: String,
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
        match url.remaining_path_parts().as_slice() {
            ["callback"] => Self::Callback(
                url.search()
                    .get("code")
                    .expect("No auth codes provided.")
                    .first()
                    .expect("No auth code provided.")
                    .clone(),
            ),
            _ => match url.remaining_hash_path_parts().as_slice() {
                ["edit-note"] => Self::EditNote,
                ["new-note"] => Self::NewNote,
                ["notes"] => Self::Notes,
                _ => Self::Main,
            },
        }
    }
}
