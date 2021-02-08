use seed::{prelude::*, *};

type Model = i32;

#[derive(Copy, Clone)]
enum Msg {
    Increment,
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
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
