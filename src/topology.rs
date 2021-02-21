use crate::osm::{OsmDocument, OsmNd, OsmNode, OsmTag, OsmWay};

#[derive(Debug)]
pub struct Topology {
    ways: Vec<Way>,
}

#[derive(Debug)]
struct Way {
    points: Vec<Point>,
    tags: Vec<Tag>,
}

#[derive(Debug)]
struct Point {
    lat: f32,
    lon: f32,
}

#[derive(Debug)]
struct Tag {
    name: String,
    value: String,
}

impl From<OsmDocument> for Topology {
    fn from(osm: OsmDocument) -> Self {
        let nodes = osm.nodes;
        Topology {
            ways: osm
                .ways
                .into_iter()
                .map(|way| Way::new(way, &nodes))
                .collect(),
        }
    }
}

impl From<OsmTag> for Tag {
    fn from(tag: OsmTag) -> Self {
        Tag {
            name: tag.k,
            value: tag.v,
        }
    }
}

impl Way {
    fn new(way: OsmWay, nodes: &[OsmNode]) -> Self {
        Way {
            tags: way.tags.into_iter().map(Tag::from).collect(),
            points: way
                .nodes
                .into_iter()
                .map(|nd| Point::new(nd, &nodes))
                .collect(),
        }
    }
}

impl Point {
    fn new(nd: OsmNd, nodes: &[OsmNode]) -> Self {
        let node = nodes
            .iter()
            .find(|node| node.id == nd.node_ref)
            .unwrap_or_else(|| panic!("Didn't find a node with id {}", nd.node_ref));

        Point {
            lat: node.lat,
            lon: node.lon,
        }
    }
}
