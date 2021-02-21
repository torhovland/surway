use log::info;
use leaflet::{LatLng, Map, Polyline, TileLayer};
use seed::{prelude::*};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CircleOptions {
    radius: f32,
}

#[derive(Serialize, Deserialize)]
struct PolylineOptions {
    color: String,
}

pub fn init(_: RenderInfo) {
    let map = Map::new("map", &JsValue::NULL);
    map.setView(&LatLng::new(63.5, 10.5), 5.0);

    TileLayer::new("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", &JsValue::NULL).addTo(&map);

    info!("Map initialized.");

    Polyline::new_with_options(
        [LatLng::new(63.25, 11.25), LatLng::new(63.75, 11.75), LatLng::new(63.5, 12.0)]
            .iter().map(JsValue::from).collect(),
        &JsValue::from_serde(&PolylineOptions { color: "red".into() }).expect("Unable to serialize polyline options")
    ).addTo(&map);
}
