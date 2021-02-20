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

fn init(_: Url, orders: &mut impl Orders<Msg>) -> Model {
    // Cannot initialize Leaflet until the map element has rendered.
    orders.after_next_render(init_map);

    Model::default()
}

fn init_map(_: RenderInfo) {
    web_sys::console::log_1(&"init_map".into());

    let map_config = MapConfig { center: [63.5, 10.09], zoom: 5.0 };
    let js_map_config = JsValue::from_serde(&map_config).expect("Unable to serialize map config");
    let layer = leaflet::TileLayer::new("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", &JsValue::NULL);
    let map = leaflet::Map::new("map", &js_map_config);
    layer.addTo(&map);
}

fn window() -> web_sys::Window {
    web_sys::window().expect("Could not get browser window.")
}

fn document() -> web_sys::Document {
    window().document().expect("Could not get browser document.")
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => *model += 1,
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
