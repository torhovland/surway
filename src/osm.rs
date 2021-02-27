use serde::Deserialize;

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
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Deserialize)]
pub struct OsmWay {
    pub id: String,
    #[serde(rename = "nd", default)]
    pub nds: Vec<OsmNd>,
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

impl OsmDocument {
    pub fn new() -> OsmDocument {
        OsmDocument {
            nodes: vec![],
            ways: vec![],
        }
    }

    fn get_node(&self, id: &str) -> &OsmNode {
        self.nodes
            .iter()
            .find(|node| node.id == id)
            .unwrap_or_else(|| panic!("Didn't find a node with id {}", id))
    }
}

impl OsmWay {
    pub fn get_points<'a>(&self, osm: &'a OsmDocument) -> Vec<&'a OsmNode> {
        self.nds
            .iter()
            .map(|nd| osm.get_node(&nd.node_ref))
            .collect()
    }
}
