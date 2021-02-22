use cfg_if::cfg_if;
use leaflet::Map;
use log::info;
use seed::{prelude::*, *};

mod map;
mod osm;
mod topology;

pub struct Model {
    map: Option<Map>,
}

enum Msg {
    SetMap(Map),
    Fetched(fetch::Result<String>),
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    // Cannot initialize Leaflet until the map element has rendered.
    orders.after_next_render(|_| {
        let map_view = map::init();
        Msg::SetMap(map_view)
    });

    orders
        .skip()
        .perform_cmd(async { Msg::Fetched(send_message().await) });

    Model { map: None }
}

fn get_request_url() -> &'static str {
    "https://www.openstreetmap.org/api/0.6/map?bbox=10.29072%2C63.39981%2C10.29426%2C63.40265"
}

async fn send_message() -> fetch::Result<String> {
    fetch(get_request_url()).await?.check_status()?.text().await
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::SetMap(map) => {
            model.map = Some(map);
        }

        Msg::Fetched(Ok(response_data)) => {
            info!("{}", response_data);
            let osm: osm::OsmDocument = quick_xml::de::from_str(&response_data)
                .expect("Unable to deserialize the OSM data");

            let topology: topology::Topology = osm.into();

            map::render_topology(&topology, &model);

            info!("{:?}", topology);
        }

        Msg::Fetched(Err(fetch_error)) => {
            error!("Fetching OSM data failed: {:#?}", fetch_error);
        }
    }
}

fn view(_: &Model) -> Node<Msg> {
    div![div![id!["map"]],]
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
