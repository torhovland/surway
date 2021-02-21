use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct OsmDocument {
    #[serde(rename = "node", default)]
    pub nodes: Vec<OsmNode>,
    #[serde(rename = "way", default)]
    pub ways: Vec<OsmWay>,
}

#[derive(Debug, Deserialize)]
pub struct OsmNode {
    pub id: String,
    pub lat: f32,
    pub lon: f32,
}

#[derive(Debug, Deserialize)]
pub struct OsmWay {
    #[serde(rename = "nd", default)]
    pub nodes: Vec<OsmNd>,
    #[serde(rename = "tag", default)]
    pub tags: Vec<OsmTag>,
}

#[derive(Debug, Deserialize)]
pub struct OsmNd {
    #[serde(rename = "ref", default)]
    pub node_ref: String,
}

#[derive(Debug, Deserialize)]
pub struct OsmTag {
    pub k: String,
    pub v: String,
}
