use bindings::GeolocationPosition;
use cfg_if::cfg_if;
use geo::{destination, BoundingBox, Coord};
use js_sys::Date;
use leaflet::{LayerGroup, Map};
use log::{error, info, warn};
use model::{Model, Note, NoteId, Route};
use osm::OsmDocument;
use rand::prelude::*;
use seed::{prelude::*, *};
use web_sys::{Element, PositionOptions, WakeLock, WakeLockSentinel, WakeLockType};

mod bindings;
mod geo;
mod map;
mod model;
mod osm;

const NOTE_STORAGE_KEY: &str = "notes";

enum Msg {
    UrlChanged(subs::UrlChanged),
    DownloadOsmChunk,
    InvalidateMapSize,
    NoteChanged(String),
    OsmFetched(fetch::Result<String>),
    Position(Coord),
    Locate(Coord),
    RandomWalk,
    SaveNote,
    NewNote,
    EditNote(NoteId),
    DeleteNote(NoteId),
    SetMapMoveHandler,
    MapMoved,
    FlipTrackPosition,
    FlipWakeLock,
    KeepWakeLockSentinel(WakeLockSentinel),
    OnTick,
}

fn main() {
    init_log();
    App::start("app", init, update, view);
}

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    init_geolocation(orders);

    let (app, msg_mapper) = (orders.clone_app(), orders.msg_mapper());
    let track_position_callback = move || {
        app.update(msg_mapper(Msg::FlipTrackPosition));
    };

    let (app, msg_mapper) = (orders.clone_app(), orders.msg_mapper());
    let wake_lock_callback = if is_wake_lock_supported() {
        Some(move || {
            app.update(msg_mapper(Msg::FlipWakeLock));
        })
    } else {
        None
    };

    orders
        .subscribe(Msg::UrlChanged) // Handle route changes.
        .notify(subs::UrlChanged(url.clone())) // Handle initial route.
        .after_next_render(move |info| {
            info!("RenderInfo: {:?}", info);

            map::init(track_position_callback, wake_lock_callback);
        }) // Cannot initialize Leaflet until the map element has rendered
        //.after_next_render(|_| Msg::SetMapMoveHandler) // Cannot initialize Leaflet until the map element has rendered.
        .after_next_render(|info| {
            info!("RenderInfo: {:?}", info);
        }); // Cannot initialize Leaflet until the map element has rendered.

    let timer_handle = orders.stream_with_handle(streams::interval(3000, || Msg::OnTick));

    // TODO: Handle like any other route
    if url.search().contains_key("random_walk") {
        orders.stream(streams::interval(5000, || Msg::RandomWalk));
    }

    // Create a random start location, so we get to init the map even if geolocation isn't available.
    let mut rng = thread_rng();
    let position = Coord {
        lat: rng.gen_range(-90.0..90.0),
        lon: rng.gen_range(-180.0..180.0),
    };

    Model {
        route: Route::from(url),
        topology_layer_group: None,
        position_layer_group: None,
        notes_layer_group: None,
        osm: OsmDocument::new(),
        position,
        track_position: true,
        osm_chunk_position: None,
        osm_chunk_radius: 500.0,
        osm_chunk_trigger_factor: 0.8,
        notes: LocalStorage::get(NOTE_STORAGE_KEY).unwrap_or_default(),
        new_note: "".into(),
        note_id: None,
        wake_lock_sentinel: None,
        timer_handle: Some(timer_handle),
    }
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            let route = Route::from(url);
            model.route = route;
        }

        Msg::DownloadOsmChunk => {
            let bbox = model.position.bbox(model.osm_chunk_radius);
            orders.perform_cmd(async move { Msg::OsmFetched(send_osm_request(&bbox).await) });
        }

        Msg::InvalidateMapSize => {
            if let Some(map) = &model.map {
                map.invalidateSize(true)
            };
        }

        Msg::NoteChanged(text) => {
            model.new_note = text;
        }

        Msg::OsmFetched(Ok(response_data)) => {
            model.osm = quick_xml::de::from_str(&response_data)
                .expect("Unable to deserialize the OSM data");

            map::render_topology_and_position(model);
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

        Msg::Position(position) => {
            if model.track_position {
                pan_to_position(model, position);
            }

            model.position = position;
            update_position(model, orders);
        }

        Msg::Locate(position) => {
            if model.track_position {
                model.track_position = false;
                flip_track_position_icon();
            }

            pan_to_position(model, position);
        }

        Msg::RandomWalk => {
            let mut rng = thread_rng();
            let bearing = rng.gen_range(0.0..360.0);
            let distance = rng.gen_range(0.0..100.0);
            model.position = destination(&model.position, bearing, distance);

            if model.track_position {
                pan_to_position(model, model.position);
            }

            info!(
                "Walking randomly for {} m at bearing {} yields new position {:?}.",
                distance, bearing, model.position
            );

            update_position(model, orders);
        }

        Msg::SaveNote => {
            let id = model.note_id.unwrap_or_else(NoteId::new);
            let time;
            let position;

            if let Some(existing_note) = model.notes.iter().find(|note| note.id == id) {
                time = existing_note.time;
                position = existing_note.position;
            } else {
                time = Date::now();
                position = model.position;
            }

            let note = Note {
                id,
                time,
                position,
                text: model.new_note.clone(),
            };

            model.notes.retain(|note| note.id != id);
            model.notes.push_front(note);

            model.note_id = None;
            model.new_note = "".into();

            LocalStorage::insert(NOTE_STORAGE_KEY, &model.notes)
                .expect("Unable to save note to LocalStorage");

            map::render_notes(model);
        }

        Msg::NewNote => {
            model.note_id = None;
            model.new_note = String::new();
            model.route = Route::EditNote;
        }

        Msg::EditNote(id) => {
            model.note_id = Some(id);
            model.new_note = model
                .notes
                .iter()
                .find(|note| note.id == id)
                .unwrap_or_else(|| panic!("Did not find a note with id {}", id))
                .text
                .clone();
            model.route = Route::EditNote;
        }

        Msg::DeleteNote(id) => {
            model.notes.retain(|note| note.id != id);

            LocalStorage::insert(NOTE_STORAGE_KEY, &model.notes)
                .expect("Unable to save note to LocalStorage");

            map::render_notes(model);
        }

        Msg::SetMap((map, topology_layer_group, position_layer_group, notes_layer_group)) => {
            model.map = Some(map);
            model.topology_layer_group = Some(topology_layer_group);
            model.position_layer_group = Some(position_layer_group);
            model.notes_layer_group = Some(notes_layer_group);
            map::set_view(model);
            map::render_topology_and_position(model);
            map::render_notes(model);
        }

        Msg::SetMapMoveHandler => {
            let (app, msg_mapper) = (orders.clone_app(), orders.msg_mapper());
            let move_map_callback = move || {
                //app.update(msg_mapper(Msg::FlipTrackPosition));
            };

            let map = model.map.as_ref().expect("Map not set.");
            map::update(map, move_map_callback);
        }

        Msg::MapMoved => {
            info!("Map moved!");
        }

        Msg::FlipTrackPosition => {
            info!("flip");
            model.track_position = !model.track_position;
            flip_track_position_icon();
        }

        Msg::FlipWakeLock => {
            if let Some(sentinel) = &model.wake_lock_sentinel {
                let _promise = sentinel.release();
                model.wake_lock_sentinel = None;
                flip_wake_lock_icon();
            } else {
                orders.skip().perform_cmd({
                    async {
                        let sentinel: WakeLockSentinel = JsCast::unchecked_into(
                            wasm_bindgen_futures::JsFuture::from(
                                wake_lock().request(WakeLockType::Screen),
                            )
                            .await
                            .expect("Unable to get wake lock result."),
                        );

                        flip_wake_lock_icon();
                        Msg::KeepWakeLockSentinel(sentinel)
                    }
                });
            }
        }

        Msg::KeepWakeLockSentinel(sentinel) => {
            model.wake_lock_sentinel = Some(sentinel);
        }

        Msg::OnTick => {
            info!("tick");
            model.timer_handle = None;

            let (app, msg_mapper) = (orders.clone_app(), orders.msg_mapper());
            let move_map_callback = move || {
                app.update(msg_mapper(Msg::FlipTrackPosition));
            };

            let map = model.map.as_ref().expect("Map not set.");
            map::update(map, move_map_callback);
        }
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![
        div![
            C!["content"],
            div![id!["map"],],
            div![
                C!["button-row"],
                a!(
                    C!["btn btn-primary"],
                    attrs! {
                        At::Href => "#notes"
                    },
                    "Notes"
                )
            ],
            view_way(model),
        ],
        div![
            C![if model.route != Route::Main {
                "modal modal-lg active"
            } else {
                "modal"
            }],
            a![C!["modal-overlay"], attrs! {At::Href => "#"}],
            div![
                C!["modal-container"],
                div![
                    C!["modal-header"],
                    a![C!["btn btn-clear float-right"], attrs! {At::Href => "#"}],
                    div![C!["modal-title h5"], route_title(model.route)]
                ],
                view_modal(model)
            ]
        ]
    ]
}

fn view_modal(model: &Model) -> Node<Msg> {
    match model.route {
        Route::Main => div![],
        Route::Notes => view_notes(model),
        Route::EditNote => view_edit_note(model),
        Route::NewNote => view_edit_note(model),
    }
}

fn view_notes(model: &Model) -> Node<Msg> {
    div![
        C!["modal-body"],
        model.notes.iter().map(|note| {
            let note_id = note.id;
            let position = note.position;
            let mut time: String = Date::new(&JsValue::from(note.time)).to_string().into();

            if let (Some(start), Some(end)) = (time.find('('), time.find(')')) {
                time.replace_range(start..=end, "");
            }

            div![
                C!["card-container"],
                div![
                    C!["card"],
                    div![
                        C!["card-header"],
                        div![
                            C!["btn-group  float-right"],
                            a![
                                C!["btn"],
                                attrs! {At::Href => "#"},
                                img![attrs! {At::Src => "icons/locate.svg"}, C!["icon"]],
                                ev(Ev::Click, move |_| Msg::Locate(position))
                            ],
                            button![
                                C!["btn"],
                                img![attrs! {At::Src => "icons/pen.svg"}, C!["icon"]],
                                ev(Ev::Click, move |_| Msg::EditNote(note_id))
                            ],
                            button![
                                C!["btn"],
                                img![attrs! {At::Src => "icons/trash.svg"}, C!["icon"]],
                                ev(Ev::Click, move |_| Msg::DeleteNote(note_id))
                            ],
                        ],
                        div![C!["card-subtitle text-gray"], time],
                    ],
                    div![C!["card-body"], p![note.text.to_string()],],
                ],
            ]
        }),
        div![
            C!["modal-footer"],
            button![
                C!["btn btn-primary"],
                "Take a note",
                ev(Ev::Click, move |_| Msg::NewNote)
            ],
        ]
    ]
}

fn view_edit_note(model: &Model) -> Node<Msg> {
    div![
        C!["modal-body"],
        textarea![
            attrs! {At::Value => model.new_note },
            input_ev(Ev::Input, Msg::NoteChanged)
        ],
        div![
            C!["modal-footer"],
            div![
                a![
                    C!["btn btn-primary"],
                    attrs! {At::Href => "#"},
                    "Save",
                    ev(Ev::Click, |_| Msg::SaveNote)
                ],
                a![C!["btn btn-link"], attrs! {At::Href => "#notes"}, "Cancel",]
            ],
        ]
    ]
}

fn route_title(route: Route) -> &'static str {
    match route {
        Route::Main => "Main",
        Route::Notes => "Notes",
        Route::EditNote => "Edit note",
        Route::NewNote => "Take a note",
    }
}

fn view_way(model: &Model) -> Node<Msg> {
    match model.find_nearest_way() {
        Some(way) => {
            div![
                C!["way-info"],
                div![
                    C!["flex-list"],
                    way.tags.iter().map(|tag| div![
                        img![attrs! {At::Src => "icons/tag.svg"}, C!["icon"]],
                        format!(" {} = {}", tag.k, tag.v),
                    ])
                ],
                div![match (way.start(&model.osm), way.end(&model.osm)) {
                    (Some(start), Some(end)) => {
                        div![
                            C!["flex-list"],
                            div![
                                img![attrs! {At::Src => "icons/ruler-green.svg"}, C!["icon"]],
                                format!(" start: {} m", start.distance(&model.position).round()),
                            ],
                            div![
                                img![attrs! {At::Src => "icons/ruler-green.svg"}, C!["icon"]],
                                format!(" end: {} m", end.distance(&model.position).round())
                            ],
                            div![
                                img![attrs! {At::Src => "icons/ruler-green.svg"}, C!["icon"]],
                                format!(
                                    " away: {} m",
                                    way.distance(&model.position, &model.osm).round()
                                )
                            ],
                        ]
                    }
                    _ => {
                        div![]
                    }
                }]
            ]
        }
        None => div![],
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

    if status.category == fetch::StatusCategory::Success {
        Ok(body)
    } else {
        Err(FetchError::StatusError(status))
    }
}

fn update_position(model: &mut Model, orders: &mut impl Orders<Msg>) {
    map::render_position(model);

    if model.is_outside_osm_trigger_box() {
        model.osm_chunk_position = Some(model.position);
        orders.send_msg(Msg::DownloadOsmChunk);
    }

    // Make sure the map is centered on our position even if the size of the map has changed
    orders.after_next_render(|_| Msg::InvalidateMapSize);
}

fn pan_to_position(model: &mut Model, position: Coord) {
    map::pan_to_position(model, position);
}

fn init_geolocation(orders: &mut impl Orders<Msg>) {
    let geolocation = window()
        .navigator()
        .geolocation()
        .expect("Unable to get geolocation.");

    let (app, msg_mapper) = (orders.clone_app(), orders.msg_mapper());

    let geo_callback = move |position: JsValue| {
        let pos: GeolocationPosition = position.into();
        let coords = pos.coords();

        app.update(msg_mapper(Msg::Position(coords.into())));
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
}

fn wake_lock() -> WakeLock {
    window().navigator().wake_lock()
}

fn is_wake_lock_supported() -> bool {
    let wake_lock_test: JsValue = JsCast::unchecked_into(wake_lock());
    wake_lock_test != JsValue::UNDEFINED
}

fn flip_track_position_icon() {
    let css_class = "icon-control-container-enabled";

    let class_list = document()
        .get_element_by_id("track-position-control-container")
        .expect("Unable to get track position control container.")
        .dyn_into::<Element>()
        .expect("Unable to get track position Element.")
        .class_list();

    if class_list.contains(css_class) {
        class_list
            .remove_1(css_class)
            .expect("Unable to remove class to track position Element.");
    } else {
        class_list
            .add_1(css_class)
            .expect("Unable to add class to track position Element.");
    }
}

fn flip_wake_lock_icon() {
    let css_class = "icon-control-container-enabled";

    let class_list = document()
        .get_element_by_id("wake-lock-control-container")
        .expect("Unable to get wake lock control container.")
        .dyn_into::<Element>()
        .expect("Unable to get wake lock Element.")
        .class_list();

    if class_list.contains(css_class) {
        class_list
            .remove_1(css_class)
            .expect("Unable to remove class to wake lock Element.");
    } else {
        class_list
            .add_1(css_class)
            .expect("Unable to add class to wake lock Element.");
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
