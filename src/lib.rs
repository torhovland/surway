use bindings::GeolocationPosition;
use cfg_if::cfg_if;
use geo::{destination, BoundingBox, Coord};
use js_sys::Date;
use leaflet::{LayerGroup, Map};
use log::{error, info, warn};
use model::{Model, Note, NoteId, OAuth2Response, Route, User};
use osm::OsmDocument;
use rand::prelude::*;
use seed::{prelude::*, *};
use urlencoding::encode;
use web_sys::{Element, PositionOptions, WakeLock, WakeLockSentinel, WakeLockType};

use crate::model::UserResponse;

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
    OsmMapFetched(fetch::Result<String>),
    OsmUserFetched(fetch::Result<User>),
    OsmNotePosted(fetch::Result<NoteId>),
    OsmAuthenticated(fetch::Result<String>),
    Position(Coord),
    Locate(Coord),
    RandomWalk,
    SaveNote,
    NewNote,
    EditNote(NoteId),
    UploadNote(NoteId),
    DeleteNote(NoteId),
    SetMap((Map, LayerGroup, LayerGroup, LayerGroup)),
    FlipTrackPosition,
    FlipWakeLock,
    KeepWakeLockSentinel(WakeLockSentinel),
}

#[wasm_bindgen(start)]
pub fn start() {
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
        .after_next_render(move |_| {
            Msg::SetMap(map::init(track_position_callback, wake_lock_callback))
        }); // Cannot initialize Leaflet until the map element has rendered.

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
        access_token: None,
        user: None,
        map: None,
        topology_layer_group: None,
        position_layer_group: None,
        notes_layer_group: None,
        osm: OsmDocument::new(),
        position,
        nearest_way_id: None,
        start_distance: None,
        end_distance: None,
        way_distance: None,
        track_position: true,
        osm_chunk_position: None,
        osm_chunk_radius: 500.0,
        osm_chunk_trigger_factor: 0.8,
        notes: LocalStorage::get(NOTE_STORAGE_KEY).unwrap_or_default(),
        new_note: "".into(),
        note_id: None,
        wake_lock_sentinel: None,
    }
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            info!("URL changed: {}", url);
            let route = Route::from(url);
            info!("Route: {:?}", route);

            if let Route::Callback(code) = route.clone() {
                info!("Auth code: {}", code);
                orders.perform_cmd(async move {
                    Msg::OsmAuthenticated(send_osm_token_request(&code).await)
                });

                model.route = Route::Main;
            } else {
                model.route = route;
            }
        }

        Msg::DownloadOsmChunk => {
            let bbox = model.position.bbox(model.osm_chunk_radius);
            orders
                .perform_cmd(async move { Msg::OsmMapFetched(send_osm_map_request(&bbox).await) });
        }

        Msg::InvalidateMapSize => {
            if let Some(map) = &model.map {
                map.invalidateSize(true)
            };
        }

        Msg::NoteChanged(text) => {
            model.new_note = text;
        }

        Msg::OsmMapFetched(Ok(response_data)) => {
            model.osm = quick_xml::de::from_str(&response_data)
                .expect("Unable to deserialize the OSM data");

            // If we haven't calculated nearest way yet, do it now
            update_position(model.position, model, orders);

            map::render_topology_and_position(model);
        }

        Msg::OsmMapFetched(Err(fetch_error)) => {
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

        Msg::OsmAuthenticated(Ok(access_token)) => {
            info!("User {} successfully authenticated.", access_token);
            let request_token = access_token.clone();
            model.access_token = Some(access_token);

            orders.perform_cmd(async move {
                Msg::OsmUserFetched(send_osm_user_request(&request_token).await)
            });
        }

        Msg::OsmAuthenticated(Err(fetch_error)) => {
            error!("OSM authentication failed: {:#?}", fetch_error);
        }

        Msg::OsmUserFetched(Ok(user)) => {
            model.user = Some(user);
        }

        Msg::OsmUserFetched(Err(fetch_error)) => {
            error!("Fetching OSM user failed: {:#?}", fetch_error);
        }

        Msg::Position(position) => {
            update_position(position, model, orders);
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
            let position = destination(&model.position, bearing, distance);

            info!(
                "Walking randomly for {} m at bearing {} yields new position {:?}.",
                distance, bearing, position
            );

            update_position(position, model, orders);
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
                uploaded: false,
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

        Msg::UploadNote(id) => {
            let note: Note = model
                .notes
                .iter()
                .find(|note| note.id == id)
                .unwrap_or_else(|| panic!("Did not find a note with id {}", id))
                .clone();

            if !note.uploaded {
                orders.perform_cmd(async { Msg::OsmNotePosted(send_osm_note_request(note).await) });
            }
        }

        Msg::OsmNotePosted(Ok(note_id)) => {
            if let Some(note) = model.notes.iter_mut().find(|note| note.id == note_id) {
                note.uploaded = true;

                LocalStorage::insert(NOTE_STORAGE_KEY, &model.notes)
                    .expect("Unable to save note to LocalStorage");
            }
        }

        Msg::OsmNotePosted(Err(fetch_error)) => {
            error!("Posting OSM note failed: {:#?}", fetch_error);
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

        Msg::FlipTrackPosition => {
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
                ),
                a!(
                    C!["btn"],
                    attrs! {
                        At::Href => format!("https://www.openstreetmap.org/oauth2/authorize?response_type=code&client_id={}&scope=read_prefs+write_notes&redirect_uri={}&state=foo",
                        oauth2_client(), oauth2_callback())
                    },
                    model
                        .user
                        .as_ref()
                        .map(|u| u.name.clone())
                        .unwrap_or_else(|| "OSM login".into())
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
                    div![C!["modal-title h5"], route_title(model.route.clone())]
                ],
                view_modal(model)
            ]
        ]
    ]
}

fn view_modal(model: &Model) -> Node<Msg> {
    match model.route {
        Route::Notes => view_notes(model),
        Route::EditNote => view_edit_note(model),
        Route::NewNote => view_edit_note(model),
        _ => div![],
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
                                C![if note.uploaded {
                                    "btn icon-enabled"
                                } else {
                                    "btn"
                                }],
                                img![attrs! {At::Src => "icons/upload.svg"}, C!["icon"]],
                                ev(Ev::Click, move |_| Msg::UploadNote(note_id))
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
        Route::Notes => "Notes",
        Route::EditNote => "Edit note",
        Route::NewNote => "Take a note",
        _ => "Surway",
    }
}

fn view_way(model: &Model) -> Node<Msg> {
    match (
        model.nearest_way_id.clone(),
        model.start_distance,
        model.end_distance,
        model.way_distance,
    ) {
        (Some(way_id), Some(start_distance), Some(end_distance), Some(way_distance)) => {
            let way = model
                .osm
                .ways
                .iter()
                .find(|w| w.id == way_id)
                .expect("No longer have the way with the expected id.");
            div![
                C!["way-info"],
                div![
                    C!["flex-list"],
                    way.tags.iter().map(|tag| div![
                        img![attrs! {At::Src => "icons/tag.svg"}, C!["icon"]],
                        format!(" {} = {}", tag.k, tag.v),
                    ])
                ],
                div![
                    C!["flex-list"],
                    div![
                        img![attrs! {At::Src => "icons/ruler-green.svg"}, C!["icon"]],
                        format!(" start: {} m", start_distance),
                    ],
                    div![
                        img![attrs! {At::Src => "icons/ruler-green.svg"}, C!["icon"]],
                        format!(" end: {} m", end_distance)
                    ],
                    div![
                        img![attrs! {At::Src => "icons/ruler-green.svg"}, C!["icon"]],
                        format!(" away: {} m", way_distance)
                    ]
                ]
            ]
        }
        _ => div![],
    }
}

async fn send_osm_map_request(bbox: &BoundingBox) -> fetch::Result<String> {
    let url = format!(
        "https://overpass-api.de/api/interpreter?data=[bbox];way[highway];(._;>;);out;&bbox={},{},{},{}",
        bbox.lower_left.lon, bbox.lower_left.lat, bbox.upper_right.lon, bbox.upper_right.lat
    );

    info!("Fetching query {}", url);

    let response = Request::new(url).fetch().await?;
    let status = response.status();
    let body = response.text().await.expect("Unable to get response text");

    if status.category == fetch::StatusCategory::Success {
        Ok(body)
    } else {
        Err(FetchError::StatusError(status))
    }
}

async fn send_osm_note_request(note: Note) -> fetch::Result<NoteId> {
    let url = format!(
        "https://api.openstreetmap.org/api/0.6/notes?lat={}&lon={}&text={}",
        note.position.lat,
        note.position.lon,
        encode(note.text.as_str())
    );

    info!("Posting note {}", url);

    let status = Request::new(url)
        .method(Method::Post)
        .fetch()
        .await?
        .status();

    if status.category == fetch::StatusCategory::Success {
        Ok(note.id)
    } else {
        Err(FetchError::StatusError(status))
    }
}

async fn send_osm_token_request(code: &str) -> fetch::Result<String> {
    let url = "https://www.openstreetmap.org/oauth2/token";

    let response = Request::new(url)
        .method(Method::Post)
        .text(format!(
            "grant_type=authorization_code&redirect_uri=http://127.0.0.1:8088/callback&code={}",
            code
        ))
        .header(Header::content_type("application/x-www-form-urlencoded"))
        .header(Header::authorization(format!("Basic {}", oauth2_auth())))
        .fetch()
        .await?;

    let status = response.status();
    let body: OAuth2Response = response.json().await.expect("Unable to get response text");

    if status.category == fetch::StatusCategory::Success {
        info!("Response: {:?}", body);
        Ok(body.access_token)
    } else {
        Err(FetchError::StatusError(status))
    }
}

async fn send_osm_user_request(access_token: &str) -> fetch::Result<User> {
    let url = "https://www.openstreetmap.org/api/0.6/user/details.json";

    let response = Request::new(url)
        .header(Header::authorization(format!("Bearer {}", access_token)))
        .fetch()
        .await?;

    let status = response.status();
    let response: UserResponse = response.json().await.expect("Unable to get user JSON");

    if status.category == fetch::StatusCategory::Success {
        info!("Response: {:?}", response);
        let user = User {
            name: response.user.display_name,
            photo: "".into(),
        };
        Ok(user)
    } else {
        Err(FetchError::StatusError(status))
    }
}

fn update_position(position: Coord, model: &mut Model, orders: &mut impl Orders<Msg>) {
    if model.nearest_way_id.is_some() && position == model.position {
        info!("Position unchanged.");
        return;
    }

    model.position = position;
    let nearest_way = model.find_nearest_way();
    let nearest_way_id = nearest_way.map(|w| w.id.clone());

    let start_position = nearest_way.map(|w| w.start(&model.osm)).flatten();
    let end_position = nearest_way.map(|w| w.end(&model.osm)).flatten();

    let start_distance = start_position.map(|p| p.distance(&model.position).round());
    let end_distance = end_position.map(|p| p.distance(&model.position).round());
    let way_distance = nearest_way.map(|w| w.distance(&model.position, &model.osm).round());

    model.nearest_way_id = nearest_way_id;
    model.start_distance = start_distance;
    model.end_distance = end_distance;
    model.way_distance = way_distance;

    if model.track_position {
        pan_to_position(model, position);
    }

    map::render_position(model);

    if model.is_outside_osm_trigger_box() {
        model.osm_chunk_position = Some(position);
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
    let css_class = "icon-enabled";

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
    let css_class = "icon-enabled";

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
        fn oauth2_callback() -> String { "http://127.0.0.1:8088/callback".into() }
        fn oauth2_client() -> String { "H_ZgxAxDxk7mvYbsd_ub9igbYZEoFNkzEB49VogyQH8".into() }
        fn oauth2_auth() -> String { "SF9aZ3hBeER4azdtdllic2RfdWI5aWdiWVpFb0ZOa3pFQjQ5Vm9neVFIODo=".into() }
    } else {
        fn init_log() {}
        fn oauth2_callback() -> String { "https://surway.hovland.xyz/callback".into() }
        fn oauth2_client() -> String { "WMCljcpGb8Gr36esjgbzodI9nZ6x49bAfqF5rWDgsBk".into() }
        fn oauth2_auth() -> String { "V01DbGpjcEdiOEdyMzZlc2pnYnpvZEk5blo2eDQ5YkFmcUY1cldEZ3NCazo=".into() }
    }
}
