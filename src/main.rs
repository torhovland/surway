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
    position_layer_group: Option<LayerGroup>,
    osm: OsmDocument,
    position: Option<Coord>,
}

enum Msg {
    SetMap((Map, LayerGroup)),
    OsmFetched(fetch::Result<String>),
    RandomWalk,
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders
        .after_next_render(|_| Msg::SetMap(map::init())) // Cannot initialize Leaflet until the map element has rendered.
        .perform_cmd(async { Msg::OsmFetched(send_osm_request().await) });

    if url.search().contains_key("random_walk") {
        orders.stream(streams::interval(100, || Msg::RandomWalk));
    }

    Model {
        map: None,
        position_layer_group: None,
        osm: OsmDocument::new(),
        position: Some(Coord {
            lat: 63.4015,
            lon: 10.2935,
        }),
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

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::SetMap((map, position_layer_group)) => {
            model.map = Some(map);
            model.position_layer_group = Some(position_layer_group);
            map::set_view(&model);
            map::render_topology(&model);
        }

        Msg::OsmFetched(Ok(response_data)) => {
            model.osm = quick_xml::de::from_str(&response_data)
                .expect("Unable to deserialize the OSM data");

            map::render_topology(&model);
        }

        Msg::OsmFetched(Err(fetch_error)) => {
            error!("Fetching OSM data failed: {:#?}", fetch_error);
        }

        Msg::RandomWalk => {
            if let Some(pos) = &model.position {
                let mut rng = thread_rng();
                let bearing = rng.gen_range(0.0..360.0);
                let distance = rng.gen_range(0.0..5.0);
                model.position = Some(destination(pos, bearing, distance));
                map::render_position(&model);
            }
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        div![id!["map"]],
        div![model.osm.ways.iter().map(|way| view_way(&model, way))],
    ]
}

fn view_way(model: &Model, way: &OsmWay) -> Node<Msg> {
    div![
        h2![&way.id],
        match &model.position {
            Some(pos) => {
                div![format!("Distance = {}", way.distance(&pos, &model.osm))]
            }
            None => {
                div![]
            }
        },
        ul![way
            .tags
            .iter()
            .map(|tag| li![format!("{} = {}", tag.k, tag.v)])],
    ]
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
    fn nearest_point_on_each_way(&self) -> Vec<(Coord, f64, &OsmWay)> {
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

    fn nearest_way(&self) -> Option<&OsmWay> {
        let nearest_points = self.nearest_point_on_each_way();

        let (_, _, way) = nearest_points.iter().min_by(|(_, x, _), (_, y, _)| {
            x.partial_cmp(y).expect("Could not compare distances")
        })?;

        Some(way)
    }
}
