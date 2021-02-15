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
    let mut options = web_sys::MutationObserverInit::new();
    options.child_list(true);

    let targetNode = document().get_element_by_id("app").expect("Could not get app node.");
    let init_map_function = Closure::wrap(Box::new(|| init_map()) as Box<dyn Fn()>);
    let observer = web_sys::MutationObserver::new(&init_map_function.as_ref().unchecked_ref())
        .expect("Could not create MutationObserver");
        
    observer.observe_with_options(&targetNode, &options);    
    init_map_function.forget();

    Model::default()
}

fn init_map() {
    let map_config = MapConfig { center: [63.5, 10.09], zoom: 5.0 };
    let js_map_config = JsValue::from_serde(&map_config).unwrap();
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
