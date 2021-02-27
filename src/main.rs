use cfg_if::cfg_if;
use leaflet::Map;
use osm::{OsmDocument, OsmWay};
use seed::{prelude::*, *};

mod map;
mod osm;

pub struct Model {
    map: Option<Map>,
    osm: OsmDocument,
}

enum Msg {
    SetMap(Map),
    OsmFetched(fetch::Result<String>),
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    // Cannot initialize Leaflet until the map element has rendered.
    orders.after_next_render(|_| {
        let map_view = map::init();
        Msg::SetMap(map_view)
    });

    orders
        .skip()
        .perform_cmd(async { Msg::OsmFetched(send_osm_request().await) });

    Model {
        map: None,
        osm: OsmDocument::new(),
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
        Msg::SetMap(map) => {
            model.map = Some(map);
            map::render_topology(&model);
        }

        Msg::OsmFetched(Ok(response_data)) => {
            let osm: OsmDocument = quick_xml::de::from_str(&response_data)
                .expect("Unable to deserialize the OSM data");
            model.osm = osm;
            map::render_topology(&model);
        }

        Msg::OsmFetched(Err(fetch_error)) => {
            error!("Fetching OSM data failed: {:#?}", fetch_error);
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![div![id!["map"]], div![model.osm.ways.iter().map(view_way)],]
}

fn view_way(way: &OsmWay) -> Node<Msg> {
    div![
        h2![&way.id],
        ul![way
            .tags
            .iter()
            .map(|tag| li![format!("{} = {}", tag.k, tag.v)])]
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
