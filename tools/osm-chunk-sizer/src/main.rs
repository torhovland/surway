use std::{fs::File, ops::Bound};
use std::{io::BufReader, thread, time};

use geo::{destination, distance, Coord};
use geo_types::Point;
use gpx::read;
use gpx::{Gpx, Track, TrackSegment};

use reqwest::StatusCode;
use stopwatch::Stopwatch;

mod geo;

#[tokio::main]
async fn main() {
    let file = File::open("gpx/wow.gpx").expect("Could not open GPX file");
    let reader = BufReader::new(file);
    let gpx: Gpx = read(reader).unwrap();
    let segment = &gpx.tracks[0].segments[0];
    let mut chunk_size = 4000.0;
    let mut previous_total_downloaded_data_size = usize::MAX;
    let mut results: Vec<ChunkSizeResult> = vec![];

    loop {
        report(&results);

        results.push(test_chunk_size(segment, chunk_size).await);
        let last_result = results.last().unwrap();

        if chunk_size < 75.0 {
            println!("Chunk size is getting small. Time to stop.");
            break;
        }

        if last_result.total_downloaded_data_size == 0 {
            println!("Last chunk size didn't download anything. Time to stop.");
            break;
        }

        if (last_result.total_downloaded_data_size as f64)
            > (previous_total_downloaded_data_size as f64) * 1.2
        {
            println!("Total download size is getting bigger. Time to stop.");
            break;
        }

        previous_total_downloaded_data_size = last_result.total_downloaded_data_size;
        chunk_size *= 0.7;
    }

    report(&results);
}

fn report(results: &Vec<ChunkSizeResult>) {
    for result in results {
        println!();
        println!("Chunk size: {} m", result.chunk_size);

        if result.failed {
            println!("Failed: {}", result.error_message);
        } else {
            println!("Request count: {}", result.request_count);
            println!(
                "Total downloaded data: {} MB",
                (result.total_downloaded_data_size as f64) / 1024.0 / 1024.0
            );
            println!(
                "Total request time: {} min",
                (result.total_request_time as f64) / 1000.0 / 60.0
            );
        }
    }

    println!();
}

struct ChunkSizeResult {
    failed: bool,
    error_message: String,
    chunk_size: f64,
    request_count: u32,
    total_downloaded_data_size: usize,
    total_request_time: i64,
}

#[derive(Debug)]
struct BoundingBox {
    lower_left: Point<f64>,
    upper_right: Point<f64>,
}

async fn test_chunk_size(segment: &TrackSegment, chunk_size: f64) -> ChunkSizeResult {
    println!("Testing chunk size {} m", chunk_size);

    let mut bbox: BoundingBox = Default::default();
    let mut trigger_download_bbox = BoundingBox::default();
    let mut request_count = 0;
    let mut total_downloaded_data_size = 0;
    let mut total_request_time = 0;
    let mut failed = false;
    let mut error_message = "".into();

    for waypoint in &segment.points {
        let point = waypoint.point();
        print!(".");

        if !intersects(&trigger_download_bbox, point) {
            println!();
            println!("We are now {} m away from center, and outside the trigger box. Time to download more OSM data.", distance(&center(&bbox), &point.into()));

            bbox = bounding_box(&point.into(), chunk_size);
            trigger_download_bbox = bounding_box(&point.into(), chunk_size / 2.0);

            println!(
                "Bounding box is {} m2. Trigger box is {} m2, or {} %.",
                area(&bbox),
                area(&trigger_download_bbox),
                area(&trigger_download_bbox) / area(&bbox) * 100.0
            );

            if zero_area(&bbox) {
                failed = true;
                error_message = format!("Empty bounding box: {:?}", bbox);
                break;
            }

            let sw = Stopwatch::start_new();
            request_count += 1;

            match send_osm_request(&bbox).await {
                Err(s) => {
                    failed = true;
                    error_message = s;
                    total_downloaded_data_size = usize::MAX;
                    break;
                }
                Ok(osm) => {
                    total_downloaded_data_size += osm.len();
                    total_request_time += sw.elapsed_ms();
                }
            };
        }
    }

    ChunkSizeResult {
        failed,
        error_message,
        chunk_size,
        request_count,
        total_downloaded_data_size,
        total_request_time,
    }
}

fn intersects(bbox: &BoundingBox, point: Point<f64>) -> bool {
    point.lat() >= bbox.lower_left.lat()
        && point.lat() <= bbox.upper_right.lat()
        && point.lng() >= bbox.lower_left.lng()
        && point.lng() <= bbox.upper_right.lng()
}

fn zero_area(bbox: &BoundingBox) -> bool {
    bbox.lower_left.lat() >= bbox.upper_right.lat()
        || bbox.lower_left.lng() >= bbox.upper_right.lng()
}

fn bounding_box(center: &Coord, radius: f64) -> BoundingBox {
    BoundingBox {
        lower_left: destination(center, 225.0, radius).into(),
        upper_right: destination(center, 45.0, radius).into(),
    }
}

fn area(bbox: &BoundingBox) -> f64 {
    let d = distance(&bbox.lower_left.into(), &bbox.upper_right.into());
    d * d / 2.0
}

fn center(bbox: &BoundingBox) -> Coord {
    let a = area(bbox).sqrt();
    let b = (2.0 * a * a).sqrt();
    destination(&bbox.lower_left.into(), 45.0, b / 2.0)
}

fn get_osm_query(bbox: &BoundingBox) -> String {
    format!(
        "way({},{},{},{})[\"highway\"]; (._;>;); out;",
        bbox.lower_left.lat(),
        bbox.lower_left.lng(),
        bbox.upper_right.lat(),
        bbox.upper_right.lng()
    )
}

// fn get_osm_request_url(bbox: &BoundingBox) -> String {
//     format!(
//         //"https://www.openstreetmap.org/api/0.6/map?bbox={}%2C{}%2C{}%2C{}",
//         "https://overpass-api.de/api/map?bbox={},{},{},{}",
//         bbox.lower_left.lng(),
//         bbox.lower_left.lat(),
//         bbox.upper_right.lng(),
//         bbox.upper_right.lat()
//     )
// }

async fn send_osm_request(bbox: &BoundingBox) -> Result<String, String> {
    let url = "https://overpass-api.de/api/interpreter";
    let query = get_osm_query(bbox);
    println!("Fetching query {}", query);

    let client = reqwest::Client::new();

    loop {
        let response = client
            .post(url)
            .header("Accept-Encoding", "gzip")
            .body(query.clone())
            .send()
            .await
            .expect("OSM request failed");
        let status = response.status();
        let body = response.text().await.expect("Unable to get response text");

        if status == StatusCode::OK {
            return Ok(body);
        }

        println!("{:?}", status);

        if status != StatusCode::TOO_MANY_REQUESTS && status != StatusCode::GATEWAY_TIMEOUT {
            return Err(format!("{}: {}", status, body));
        }

        println!("Waiting 30 seconds ...");
        thread::sleep(time::Duration::from_secs(30));

        println!("Retrying...");
    }
}

impl From<geo_types::Point<f64>> for Coord {
    fn from(node: Point<f64>) -> Self {
        Coord {
            lat: node.lat(),
            lon: node.lng(),
        }
    }
}

impl From<Coord> for geo_types::Point<f64> {
    fn from(node: Coord) -> Self {
        (node.lon, node.lat).into()
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        BoundingBox {
            lower_left: (0.0, 0.0).into(),
            upper_right: (0.0, 0.0).into(),
        }
    }
}
