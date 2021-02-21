use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct OsmDocument {
    #[serde(rename = "node", default)]
    nodes: Vec<OsmNode>,
    #[serde(rename = "way", default)]
    ways: Vec<OsmWay>,
}

#[derive(Debug, Deserialize)]
struct OsmNode {
    id: String,
    lat: f32,
    lon: f32,
}

#[derive(Debug, Deserialize)]
struct OsmWay {
    #[serde(rename = "nd", default)]
    nodes: Vec<Nd>,
    #[serde(rename = "tag", default)]
    tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
struct Nd {
    #[serde(rename = "ref", default)]
    refs: String,
}

#[derive(Debug, Deserialize)]
struct Tag {
    k: String,
    v: String,
}
