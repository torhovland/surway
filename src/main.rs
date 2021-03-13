use cfg_if::cfg_if;
use geo::{destination, Coord};
use leaflet::{LayerGroup, Map};
use osm::{OsmDocument, OsmWay};
use rand::prelude::*;
use seed::{prelude::*, *};

mod geo;
mod map;
mod osm;

pub struct Model {
    map: Option<Map>,
    topology_layer_group: Option<LayerGroup>,
    position_layer_group: Option<LayerGroup>,
    osm: OsmDocument,
    position: Option<Coord>,
    osm_chunk_position: Option<Coord>,
    osm_chunk_radius: f64,
    osm_chunk_trigger_factor: f64,
}

enum Msg {
    SetMap((Map, LayerGroup, LayerGroup)),
    InvalidateMapSize,
    OsmFetched(fetch::Result<String>),
    RandomWalk,
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders
        .after_next_render(|_| Msg::SetMap(map::init())) // Cannot initialize Leaflet until the map element has rendered.
        .perform_cmd(async { Msg::OsmFetched(send_osm_request().await) });

    if url.search().contains_key("random_walk") {
        orders.stream(streams::interval(1000, || Msg::RandomWalk));
    }

    Model {
        map: None,
        topology_layer_group: None,
        position_layer_group: None,
        osm: OsmDocument::new(),
        position: Some(Coord {
            lat: 63.4015,
            lon: 10.2935,
        }),
        osm_chunk_position: Some(Coord {
            lat: 63.4015,
            lon: 10.2935,
        }),
        osm_chunk_radius: 100.0,
        osm_chunk_trigger_factor: 0.5,
    }
}

fn get_osm_request_url() -> &'static str {
    "https://www.openstreetmap.org/api/0.6/map?bbox=10.29072%2C63.39981%2C10.29426%2C63.40265"
}

async fn send_osm_request() -> fetch::Result<String> {
    fetch(get_osm_request_url())
        .await?
        .check_status()?
        .text()
        .await
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SetMap((map, topology_layer_group, position_layer_group)) => {
            model.map = Some(map);
            model.topology_layer_group = Some(topology_layer_group);
            model.position_layer_group = Some(position_layer_group);
            map::set_view(&model);
            map::render_topology(&model);
        }

        Msg::InvalidateMapSize => {
            if let Some(map) = &model.map {
                map.invalidateSize(true)
            };
        }

        Msg::OsmFetched(Ok(response_data)) => {
            model.osm = quick_xml::de::from_str(&response_data)
                .expect("Unable to deserialize the OSM data");

            log!("Rendering a new OSM topology.");
            map::render_topology(&model);
        }

        Msg::OsmFetched(Err(fetch_error)) => {
            error!("Fetching OSM data failed: {:#?}", fetch_error);
        }

        Msg::RandomWalk => {
            if let Some(pos) = &model.position {
                let mut rng = thread_rng();
                let bearing = rng.gen_range(0.0..360.0);
                let distance = rng.gen_range(0.0..50.0);
                model.position = Some(destination(pos, bearing, distance));
                map::pan_to_position(&model);
                map::render_position(&model);

                if model.is_outside_osm_trigger_box() {
                    log!("Outside OSM trigger box. Initiating download.");
                    model.osm_chunk_position = model.position;
                    map::render_topology(&model);
                }

                // Make sure the map is centered on our position even if the size of the map has changed
                orders.after_next_render(|_| Msg::InvalidateMapSize);
            }
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![C!["content"], div![id!["map"]], view_way(&model),]
}

fn view_way(model: &Model) -> Node<Msg> {
    match model.find_nearest_way() {
        Some(way) => {
            div![
                C!["way-info"],
                ul![way
                    .tags
                    .iter()
                    .map(|tag| li![format!("{} = {}", tag.k, tag.v)])],
                match &model.position {
                    Some(pos) => {
                        div![
                            div![format!(
                                "Distance away = {} m",
                                way.distance(&pos, &model.osm).round()
                            )],
                            match (way.start(&model.osm), way.end(&model.osm)) {
                                (Some(start), Some(end)) => {
                                    div![
                                        div![format!(
                                            "Distance to start = {} m",
                                            start.distance(&pos).round()
                                        )],
                                        div![format!(
                                            "Distance to end = {} m",
                                            end.distance(&pos).round()
                                        )]
                                    ]
                                }
                                _ => {
                                    div![]
                                }
                            }
                        ]
                    }
                    None => {
                        div![]
                    }
                },
            ]
        }
        None => div![],
    }
}

cfg_if! {
    if #[cfg(debug_assertions)] {
        fn init_log() {
            use log::Level;
            console_log::init_with_level(Level::Trace).expect("error initializing log");
        }
    } else {
        fn init_log() {}
    }
}

fn main() {
    init_log();
    App::start("app", init, update, view);
}

impl Model {
    fn find_nearest_point_on_each_way(&self) -> Vec<(Coord, f64, &OsmWay)> {
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
                            let destination = geo::nearest_point(&a.into(), &b.into(), pos);
                            let distance = geo::distance(pos, &destination);
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

    fn find_nearest_way(&self) -> Option<&OsmWay> {
        let nearest_points = self.find_nearest_point_on_each_way();

        let (_, _, way) = nearest_points.iter().min_by(|(_, x, _), (_, y, _)| {
            x.partial_cmp(y).expect("Could not compare distances")
        })?;

        Some(way)
    }

    fn is_outside_osm_trigger_box(&self) -> bool {
        if let (Some(pos), Some(chunk_pos)) = (&self.position, &self.osm_chunk_position) {
            let radius = self.osm_chunk_radius * self.osm_chunk_trigger_factor;
            let north = destination(chunk_pos, 0.0, radius);
            let east = destination(chunk_pos, 90.0, radius);
            let south = destination(chunk_pos, 180.0, radius);
            let west = destination(chunk_pos, 270.0, radius);

            pos.lat > north.lat || pos.lon > east.lon || pos.lat < south.lat || pos.lon < west.lon
        } else {
            false
        }
    }
}
