use wasm_bindgen::prelude::*;
use web_sys::console;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(module = "/js/defined-in-js.js")]
extern "C" {
    type MyClass;
    type GeoLocator;

    #[wasm_bindgen(constructor)]
    fn new() -> GeoLocator;

    #[wasm_bindgen(method, getter)]
    fn latitude(this: &GeoLocator) -> f64;

    #[wasm_bindgen(method, getter)]
    fn longitude(this: &GeoLocator) -> f64;

    #[wasm_bindgen(method)]
    fn locate(this: &GeoLocator) -> String;
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    console::log_1(&JsValue::from_str(&format!("Hello, {}!", name)));
}

use cfg_if::cfg_if;
use geo::{destination, BoundingBox, Coord};
use leaflet::{LayerGroup, Map};
use log::{error, info};
use osm::{OsmDocument, OsmWay};
use rand::prelude::*;
use seed::{fetch::StatusCategory, prelude::*, *};
use serde::Deserialize;

mod geo;
mod map;
mod osm;

use wasm_bindgen::prelude::*;

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
    Increment,
}

#[derive(Debug, Deserialize)]
pub struct GeolocationCoordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Deserialize)]
pub struct GeolocationPosition {
    pub coords: GeolocationCoordinates,
}

fn geo_callback(position: JsValue) {
    // let pos: GeolocationPosition =
    //     JsValue::into_serde(&position).expect("Unable to deserialize GeolocationPosition.");
    //info!("Geo callback: {:?}", position);
    unsafe {
        let s = js_sys::JSON::stringify(&position).expect("Unable to stringify JSON");
        info!("Geo callback: {:?}", s);
    }
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    let position = Coord {
        lat: 63.4015,
        lon: 10.2935,
    };

    let radius = 500.0;
    let bbox = position.bbox(radius);

    orders.after_next_render(|_| Msg::SetMap(map::init())); // Cannot initialize Leaflet until the map element has rendered.

    orders.perform_cmd(async move { Msg::OsmFetched(send_osm_request(&bbox).await) });

    if url.search().contains_key("random_walk") {
        orders.stream(streams::interval(1000, || Msg::RandomWalk));
    }

    Model {
        map: None,
        topology_layer_group: None,
        position_layer_group: None,
        osm: OsmDocument::new(),
        position: Some(position),
        osm_chunk_position: Some(position),
        osm_chunk_radius: radius,
        osm_chunk_trigger_factor: 0.8,
    }
}

// fn get_osm_request_url(position: Coord) -> &'static str {
//     "https://www.openstreetmap.org/api/0.6/map?bbox=10.29072%2C63.39981%2C10.29426%2C63.40265"
// }

// async fn send_osm_request(bbox: Coord) -> fetch::Result<String> {
//     fetch(get_osm_request_url(position))
//         .await?
//         .check_status()?
//         .text()
//         .await
// }

fn get_osm_request_url(bbox: &BoundingBox) -> String {
    format!(
        "https://overpass-api.de/api/interpreter?data=[bbox];way[highway];(._;>;);out;&bbox={},{},{},{}",
        bbox.lower_left.lon, bbox.lower_left.lat, bbox.upper_right.lon, bbox.upper_right.lat
    )
}

async fn send_osm_request(bbox: &BoundingBox) -> fetch::Result<String> {
    // https://overpass-api.de/api/interpreter?data=[bbox];way[highway];(._;>;);out;&bbox=10.29149140339605,63.40060067963628,10.295508596604192,63.402399320363706
    // https://overpass-api.de/api/interpreter?data=[bbox];way[highway];(._;>;);out;&bbox=10.29,63.40,10.30,63.42
    let url = get_osm_request_url(bbox);
    info!("Fetching {}", url);

    let response = Request::new(url).fetch().await.expect("OSM request failed");
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
                let bbox = pos.bbox(model.osm_chunk_radius);

                let mut rng = thread_rng();
                let bearing = rng.gen_range(0.0..360.0);
                let distance = rng.gen_range(0.0..200.0);
                model.position = Some(destination(pos, bearing, distance));

                map::pan_to_position(&model);
                map::render_position(&model);

                if model.is_outside_osm_trigger_box() {
                    info!("Outside OSM trigger box. Initiating download.");
                    model.osm_chunk_position = model.position;
                    // map::render_topology(&model);

                    orders
                        .perform_cmd(async move { Msg::OsmFetched(send_osm_request(&bbox).await) });
                }

                // Make sure the map is centered on our position even if the size of the map has changed
                orders.after_next_render(|_| Msg::InvalidateMapSize);
            }
        }

        Msg::Increment => {
            let geo_locator = GeoLocator::new();
            geo_locator.locate();
            info!("{}", &geo_locator.latitude());
            info!("{}", &geo_locator.longitude());

            // let window = web_sys::window().expect("Unable to get browser window.");
            // let navigator = window.navigator();
            // let geolocation = navigator.geolocation().expect("Unable to get geolocation.");
            // let geo_callback_function =
            //     Closure::wrap(Box::new(geo_callback) as Box<dyn FnMut(JsValue)>);
            // geolocation
            //     .get_current_position(geo_callback_function.as_ref().unchecked_ref())
            //     .expect("Unable to get position.");
            // geo_callback_function.forget();
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        C!["content"],
        div![id!["map"]],
        button!["Locate!", ev(Ev::Click, |_| Msg::Increment),],
        view_way(&model),
    ]
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

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // Your code goes here!
    console::log_1(&JsValue::from_str("Hello world!"));

    let geo_locator = GeoLocator::new();
    geo_locator.locate();

    App::start("app", init, update, view);

    Ok(())
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
            let bbox = chunk_pos.bbox(radius);

            pos.lat > bbox.upper_right.lat
                || pos.lon > bbox.upper_right.lon
                || pos.lat < bbox.lower_left.lat
                || pos.lon < bbox.lower_left.lon
        } else {
            false
        }
    }
}
