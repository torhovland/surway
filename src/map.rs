use crate::{geo::Coord, osm::OsmNode, Model};
use leaflet::{Circle, LatLng, Map, Polyline, TileLayer};
use seed::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PolylineOptions {
    color: String,
    weight: u32,
}

#[derive(Serialize, Deserialize)]
struct CircleOptions {
    radius: f64,
}

pub fn init() -> Map {
    let map = Map::new("map", &JsValue::NULL);

    TileLayer::new(
        "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
        &JsValue::NULL,
    )
    .addTo(&map);

    map
}

pub fn set_view(model: &Model) {
    if let (Some(map), Some(position)) = (&model.map, &model.position) {
        map.setView(&position.into(), 17.0);
    }
}

pub fn render_topology(model: &Model) {
    if let Some(map) = &model.map {
        for way in model.osm.ways.iter() {
            Polyline::new_with_options(
                way.points(&model.osm)
                    .into_iter()
                    .map(LatLng::from)
                    .map(JsValue::from)
                    .collect(),
                &JsValue::from_serde(&PolylineOptions {
                    color: if model.nearest_way().id == way.id {
                        "blue"
                    } else {
                        "green"
                    }
                    .into(),
                    weight: 2,
                })
                .expect("Unable to serialize polyline options"),
            )
            .addTo(&map);
        }

        if let Some(pos) = &model.position {
            for (destination, _, _) in model.nearest_point_on_each_way().iter() {
                Polyline::new_with_options(
                    vec![pos, destination]
                        .into_iter()
                        .map(LatLng::from)
                        .map(JsValue::from)
                        .collect(),
                    &JsValue::from_serde(&PolylineOptions {
                        color: "red".into(),
                        weight: 1,
                    })
                    .expect("Unable to serialize polyline options"),
                )
                .addTo(&map);
            }
        }
    }
}

pub fn render_position(model: &Model) {
    if let (Some(map), Some(position)) = (&model.map, &model.position) {
        Circle::new_with_options(
            &LatLng::from(position),
            &JsValue::from_serde(&CircleOptions { radius: 3.5 })
                .expect("Unable to serialize circle options"),
        )
        .addTo(&map);
    }
}

impl From<&Coord> for LatLng {
    fn from(coord: &Coord) -> Self {
        LatLng::new(coord.lat, coord.lon)
    }
}

impl From<&OsmNode> for LatLng {
    fn from(node: &OsmNode) -> Self {
        LatLng::new(node.lat, node.lon)
    }
}
