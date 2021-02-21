use seed::{prelude::*, *};

mod map;
mod osm;
mod topology;

#[macro_use]
mod util;

type Model = i32;

enum Msg {
    Increment,
    Fetched(fetch::Result<String>),
}

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    // Cannot initialize Leaflet until the map element has rendered.
    orders.after_next_render(map::init);

    orders.skip().perform_cmd({
        async { Msg::Fetched(send_message().await) }
    });

    Model::default()
}

fn get_request_url() -> &'static str {
    "https://www.openstreetmap.org/api/0.6/map?bbox=10.29072%2C63.39981%2C10.29426%2C63.40265"
}

async fn send_message() -> fetch::Result<String> {
    fetch(get_request_url())
        .await?
        .check_status()?
        .text()
        .await
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => *model += 1,

        Msg::Fetched(Ok(response_data)) => {
            console_log!("{}", response_data);
            let osm: osm::OsmDocument = quick_xml::de::from_str(&response_data).expect("Unable to deserialize the OSM data");
            let topology: topology::Topology = osm.into(); 
            console_log!("{:?}", topology);
        }

        Msg::Fetched(Err(fetch_error)) => {
            console_log!("Fetching OSM data failed: {:#?}", fetch_error);
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        div![id!["map"]],
        "This is a counter: ",
        button![model, ev(Ev::Click, |_| Msg::Increment),],
    ]
}

fn main() {
    App::start("app", init, update, view);
}
