use crate::{
    topology::{Point, Topology},
    Model,
};
use leaflet::{LatLng, Map, Polyline, TileLayer};
use log::info;
use seed::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CircleOptions {
    radius: f32,
}

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

    info!("Map initialized.");

    map
}

pub fn render_topology(topology: &Topology, model: &Model) {
    match &model.map {
        None => {}
        Some(map) => {
            for way in topology.ways.iter() {
                Polyline::new_with_options(
                    way.points
                        .iter()
                        .map(Point::to_lat_lng)
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
}
