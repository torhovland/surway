use bindings::GeolocationPosition;
use cfg_if::cfg_if;
use geo::{destination, BoundingBox, Coord};
use leaflet::{LayerGroup, Map};
use log::{error, info, warn};
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
    DownloadOsmChunk,
    InvalidateMapSize,
    OsmFetched(fetch::Result<String>),
    Position(f64, f64),
    RandomWalk,
    SetMap((Map, LayerGroup, LayerGroup)),
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    let (app, msg_mapper) = (orders.clone_app(), orders.msg_mapper());

    let geolocation = web_sys::window()
        .expect("Unable to get browser window.")
        .navigator()
        .geolocation()
        .expect("Unable to get geolocation.");

    let geo_callback = move |position: JsValue| {
        let pos: GeolocationPosition = position.into();
        let coords = pos.coords();

        app.update(msg_mapper(Msg::Position(
            coords.latitude(),
            coords.longitude(),
        )));
    };

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
        orders.stream(streams::interval(8000, || Msg::RandomWalk));
    }

    // Create a random start location, so we get to init the map even if geolocation isn't available.
    let mut rng = thread_rng();
    let position = Coord {
        lat: rng.gen_range(-90.0..90.0),
        lon: rng.gen_range(-180.0..180.0),
    };

    Model {
        map: None,
        topology_layer_group: None,
        position_layer_group: None,
        osm: OsmDocument::new(),
        position,
        osm_chunk_position: None,
        osm_chunk_radius: 500.0,
        osm_chunk_trigger_factor: 0.8,
    }
}

async fn send_osm_request(bbox: &BoundingBox) -> fetch::Result<String> {
    let url = format!(
        "https://overpass-api.de/api/interpreter?data=[bbox];way[highway];(._;>;);out;&bbox={},{},{},{}",
        bbox.lower_left.lon, bbox.lower_left.lat, bbox.upper_right.lon, bbox.upper_right.lat
    );

    info!("Fetching query {}", url);

    let response = Request::new(url).fetch().await.expect("OSM request failed");
    let status = response.status();
    let body = response.text().await.expect("Unable to get response text");

    if status.category == StatusCategory::Success {
        return Ok(body);
    } else {
        return Err(FetchError::StatusError(status));
    }
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::DownloadOsmChunk => {
            let bbox = model.position.bbox(model.osm_chunk_radius);
            orders.perform_cmd(async move { Msg::OsmFetched(send_osm_request(&bbox).await) });
        }

        Msg::InvalidateMapSize => {
            if let Some(map) = &model.map {
                map.invalidateSize(true)
            };
        }

        Msg::OsmFetched(Ok(response_data)) => {
            model.osm = quick_xml::de::from_str(&response_data)
                .expect("Unable to deserialize the OSM data");

            map::render_topology_and_position(&model);
        }

        Msg::OsmFetched(Err(fetch_error)) => {
            if let FetchError::StatusError(status) = &fetch_error {
                if status.code == 429 || status.code == 504 {
                    const SECONDS: u32 = 10;
                    warn!("Server is busy. Retrying in {} seconds.", SECONDS);
                    orders.perform_cmd(cmds::timeout(SECONDS * 1000, || Msg::DownloadOsmChunk));
                    return;
                }
            }
            error!("Fetching OSM data failed: {:#?}", fetch_error);
        }

        Msg::Position(lat, lon) => {
            model.position = Coord { lat, lon };
            handle_new_position(model, orders);
        }

        Msg::RandomWalk => {
            let mut rng = thread_rng();
            let bearing = rng.gen_range(0.0..360.0);
            let distance = rng.gen_range(0.0..200.0);
            model.position = destination(&model.position, bearing, distance);
            handle_new_position(model, orders);
        }

        Msg::SetMap((map, topology_layer_group, position_layer_group)) => {
            model.map = Some(map);
            model.topology_layer_group = Some(topology_layer_group);
            model.position_layer_group = Some(position_layer_group);
            map::set_view(&model);
            map::render_topology_and_position(&model);
        }
    }
}

fn handle_new_position(model: &mut Model, orders: &mut impl Orders<Msg>) {
    map::pan_to_position(&model);
    map::render_position(&model);

    if model.is_outside_osm_trigger_box() {
        model.osm_chunk_position = Some(model.position);
        orders.send_msg(Msg::DownloadOsmChunk);
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
                div![
                    div![format!(
                        "Distance away = {} m",
                        way.distance(&model.position, &model.osm).round()
                    )],
                    match (way.start(&model.osm), way.end(&model.osm)) {
                        (Some(start), Some(end)) => {
                            div![
                                div![format!(
                                    "Distance to start = {} m",
                                    start.distance(&model.position).round()
                                )],
                                div![format!(
                                    "Distance to end = {} m",
                                    end.distance(&model.position).round()
                                )]
                            ]
                        }
                        _ => {
                            div![]
                        }
                    }
                ]
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
