use crate::{osm::OsmNode, Model};
use leaflet::{LatLng, Map, Polyline, TileLayer};
use seed::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PolylineOptions {
    color: String,
    weight: u32,
}

pub fn init() -> Map {
    let map = Map::new("map", &JsValue::NULL);
    map.setView(&LatLng::new(63.401, 10.295), 17.0);

    TileLayer::new(
        "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
        &JsValue::NULL,
    )
    .addTo(&map);

    map
}

pub fn render_topology(model: &Model) {
    if let Some(map) = &model.map {
        for way in model.osm.ways.iter() {
            Polyline::new_with_options(
                way.points(&model.osm)
                    .map(LatLng::from)
                    .map(JsValue::from)
                    .collect(),
                &JsValue::from_serde(&PolylineOptions {
                    color: "blue".into(),
                    weight: 2,
                })
                .expect("Unable to serialize polyline options"),
            )
            .addTo(&map);
        }
    }
}

impl From<&OsmNode> for LatLng {
    fn from(node: &OsmNode) -> Self {
        LatLng::new(node.lat, node.lon)
    }
}
