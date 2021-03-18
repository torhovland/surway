use bindings::GeolocationPosition;
use cfg_if::cfg_if;
use geo::{destination, BoundingBox, Coord};
use leaflet::{LayerGroup, Map};
use log::{error, info};
use model::Model;
use osm::OsmDocument;
use rand::prelude::*;
use seed::{fetch::StatusCategory, prelude::*, *};
use web_sys::PositionOptions;

mod bindings;
mod geo;
mod map;
mod model;
mod osm;

enum Msg {
    SetMap((Map, LayerGroup, LayerGroup)),
    InvalidateMapSize,
    OsmFetched(fetch::Result<String>),
    Position(f64, f64),
    RandomWalk,
    Increment,
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    let (app, msg_mapper) = (orders.clone_app(), orders.msg_mapper());

    let mut geo_callback = move |position: JsValue| {
        let pos = JsCast::unchecked_into::<GeolocationPosition>(position);
        let coords = pos.coords();
        info!(
            "Latitude: {}. Longitude: {}.",
            coords.latitude(),
            coords.longitude()
        );

        app.update(msg_mapper(Msg::Position(
            coords.latitude(),
            coords.longitude(),
        )));
    };

    let window = web_sys::window().expect("Unable to get browser window.");
    let navigator = window.navigator();
    let geolocation = navigator.geolocation().expect("Unable to get geolocation.");
    let geo_callback_function = Closure::wrap(Box::new(geo_callback) as Box<dyn FnMut(JsValue)>);

    let mut options = PositionOptions::new();
    options.enable_high_accuracy(true);

    geolocation
        .watch_position_with_error_callback_and_options(
            geo_callback_function.as_ref().unchecked_ref(),
            None,
            &options,
        )
        .expect("Unable to get position.");
    geo_callback_function.forget();

    orders.after_next_render(|_| Msg::SetMap(map::init())); // Cannot initialize Leaflet until the map element has rendered.

    if url.search().contains_key("random_walk") {
        orders.stream(streams::interval(1000, || Msg::RandomWalk));
    }

    Model {
        map: None,
        topology_layer_group: None,
        position_layer_group: None,
        osm: OsmDocument::new(),
        position: None,
        osm_chunk_position: None,
        osm_chunk_radius: 500.0,
        osm_chunk_trigger_factor: 0.8,
    }
}

fn get_osm_query(bbox: &BoundingBox) -> String {
    format!(
        "way({},{},{},{})[\"highway\"]; (._;>;); out;",
        bbox.lower_left.lat, bbox.lower_left.lon, bbox.upper_right.lat, bbox.upper_right.lon
    )
}

async fn send_osm_request(bbox: &BoundingBox) -> fetch::Result<String> {
    let url = "https://overpass-api.de/api/interpreter";
    let query = get_osm_query(bbox);
    info!("Fetching query {}", query);

    let response = Request::new(url)
        .method(Method::Post)
        .text(query)
        .fetch()
        .await
        .expect("OSM request failed");
    let status = response.status();
    let body = response.text().await.expect("Unable to get response text");

    if status.category == StatusCategory::Success {
        return Ok(body);
    }
    // else if status.code == 429 && status.code == 504 {
    //     return Err(FetchError::StatusError(status));
    // }
    else {
        return Err(FetchError::StatusError(status));
    }

    //     info!("Waiting 30 seconds ...");
    //     thread::sleep(time::Duration::from_secs(30));

    //     info!("Retrying...");
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::SetMap((map, topology_layer_group, position_layer_group)) => {
            info!("SetMap");
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

            info!("Rendering a new OSM topology.");
            map::render_topology(&model);
        }

        Msg::OsmFetched(Err(fetch_error)) => {
            error!("Fetching OSM data failed: {:#?}", fetch_error);
        }

        Msg::RandomWalk => {
            if let Some(pos) = &model.position {
                let mut rng = thread_rng();
                let bearing = rng.gen_range(0.0..360.0);
                let distance = rng.gen_range(0.0..200.0);
                model.position = Some(destination(pos, bearing, distance));
                render_position(model, orders);
            }
        }

        Msg::Increment => {}

        Msg::Position(lat, lon) => {
            info!("Position");
            let is_first = model.position.is_none();
            model.position = Some(Coord { lat, lon });

            if is_first {
                map::set_view(&model);
            }

            render_position(model, orders);
        }
    }
}

fn render_position(model: &mut Model, orders: &mut impl Orders<Msg>) {
    map::pan_to_position(&model);
    map::render_position(&model);

    if model.is_outside_osm_trigger_box() {
        info!("Outside OSM trigger box. Initiating download.");
        model.osm_chunk_position = model.position;
        map::render_topology(&model);

        let bbox = model.position.unwrap().bbox(model.osm_chunk_radius);
        orders.perform_cmd(async move { Msg::OsmFetched(send_osm_request(&bbox).await) });
    }

    // Make sure the map is centered on our position even if the size of the map has changed
    orders.after_next_render(|_| Msg::InvalidateMapSize);
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
