use seed::{prelude::*, *};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct MapConfig {
    pub center: [f32; 2],
    pub zoom: f32,
}

type Model = i32;

#[derive(Copy, Clone)]
enum Msg {
    Increment,
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    let map_config = MapConfig { center: [63.5, 10.09], zoom: 5.0 };
    let js_map_config = JsValue::from_serde(&map_config).unwrap();
    let layer = leaflet::TileLayer::new("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", &JsValue::NULL);
    let map = leaflet::Map::new("map", &js_map_config);
    layer.addTo(&map);

    Model::default()
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => *model += 1,
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        "This is a counter: ",
        button![model, ev(Ev::Click, |_| Msg::Increment),],
    ]
}

fn main() {
    App::start("app", init, update, view);
}
