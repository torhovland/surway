use crate::{
    geo::{destination, Coord},
    osm::OsmNode,
    Model,
};
use leaflet::{Circle, LatLng, LatLngBounds, LayerGroup, Map, Polyline, Rectangle, TileLayer};
use seed::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct LineOptions {
    color: String,
    weight: u32,
    fillOpacity: f64,
}

#[derive(Serialize, Deserialize)]
struct CircleOptions {
    radius: f64,
}

pub fn init() -> (Map, LayerGroup, LayerGroup) {
    let map = Map::new("map", &JsValue::NULL);

    let topology_layer_group = LayerGroup::new();
    topology_layer_group.addTo(&map);

    let position_layer_group = LayerGroup::new();
    position_layer_group.addTo(&map);

    TileLayer::new(
        "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
        &JsValue::NULL,
    )
    .addTo(&map);

    (map, topology_layer_group, position_layer_group)
}

pub fn set_view(model: &Model) {
    if let (Some(map), Some(position)) = (&model.map, &model.position) {
        map.setView(&position.into(), 19.0);
    }
}

pub fn pan_to_position(model: &Model) {
    if let (Some(map), Some(position)) = (&model.map, &model.position) {
        map.panTo(&position.into());
    }
}

pub fn render_topology(model: &Model) {
    if let (
        Some(map),
        Some(topology_layer_group),
        Some(position_layer_group),
        Some(chunk_position),
    ) = (
        &model.map,
        &model.topology_layer_group,
        &model.position_layer_group,
        &model.osm_chunk_position,
    ) {
        topology_layer_group.clearLayers();

        topology_layer_group.addLayer(&Rectangle::new_with_options(
            &bbox(chunk_position, model.osm_chunk_radius),
            &JsValue::from_serde(&LineOptions {
                color: "red".into(),
                weight: 1,
                fillOpacity: 0.0,
            })
            .expect("Unable to serialize rectangle options"),
        ));

        topology_layer_group.addLayer(&Rectangle::new_with_options(
            &bbox(
                chunk_position,
                model.osm_chunk_radius * model.osm_chunk_trigger_factor,
            ),
            &JsValue::from_serde(&LineOptions {
                color: "orange".into(),
                weight: 1,
                fillOpacity: 0.0,
            })
            .expect("Unable to serialize rectangle options"),
        ));

        for way in model.osm.ways.iter() {
            topology_layer_group.addLayer(&Polyline::new_with_options(
                way.points(&model.osm)
                    .into_iter()
                    .map(LatLng::from)
                    .map(JsValue::from)
                    .collect(),
                &JsValue::from_serde(&LineOptions {
                    color: "green".into(),
                    weight: 2,
                    fillOpacity: 0.0,
                })
                .expect("Unable to serialize polyline options"),
            ));
        }

        topology_layer_group.addTo(&map);
        position_layer_group.addTo(&map);
    }
}

pub fn render_position(model: &Model) {
    if let (Some(map), Some(topology_layer_group), Some(position_layer_group), Some(position)) = (
        &model.map,
        &model.topology_layer_group,
        &model.position_layer_group,
        &model.position,
    ) {
        position_layer_group.clearLayers();

        position_layer_group.addLayer(&Circle::new_with_options(
            &LatLng::from(position),
            &JsValue::from_serde(&CircleOptions { radius: 3.5 })
                .expect("Unable to serialize circle options"),
        ));

        if let Some(nearest) = model.find_nearest_way() {
            position_layer_group.addLayer(&Polyline::new_with_options(
                nearest
                    .points(&model.osm)
                    .into_iter()
                    .map(LatLng::from)
                    .map(JsValue::from)
                    .collect(),
                &JsValue::from_serde(&LineOptions {
                    color: "blue".into(),
                    weight: 2,
                    fillOpacity: 0.0,
                })
                .expect("Unable to serialize polyline options"),
            ));
        }

        for (destination, _, _) in model.find_nearest_point_on_each_way().iter() {
            position_layer_group.addLayer(&Polyline::new_with_options(
                vec![position, destination]
                    .into_iter()
                    .map(LatLng::from)
                    .map(JsValue::from)
                    .collect(),
                &JsValue::from_serde(&LineOptions {
                    color: "purple".into(),
                    weight: 1,
                    fillOpacity: 0.0,
                })
                .expect("Unable to serialize polyline options"),
            ));
        }

        topology_layer_group.addTo(&map);
        position_layer_group.addTo(&map);
    }
}

fn bbox(position: &Coord, radius: f64) -> LatLngBounds {
    let north = destination(position, 0.0, radius);
    let east = destination(position, 90.0, radius);
    let west = destination(position, 270.0, radius);
    let south = destination(position, 180.0, radius);

    LatLngBounds::new(
        &LatLng::new(south.lat, west.lon),
        &LatLng::new(north.lat, east.lon),
    )
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
