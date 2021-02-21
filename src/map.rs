use seed::{prelude::*};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct MapConfig {
    pub center: [f32; 2],
    pub zoom: f32,
}

pub fn init(_: RenderInfo) {
    let map_config = MapConfig { center: [63.5, 10.09], zoom: 5.0 };
    let js_map_config = JsValue::from_serde(&map_config).expect("Unable to serialize map config");
    let layer = leaflet::TileLayer::new("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", &JsValue::NULL);
    let map = leaflet::Map::new("map", &js_map_config);
    layer.addTo(&map);
}
