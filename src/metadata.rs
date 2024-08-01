use color_eyre::eyre::Context;
use serde::Deserialize;
use std::collections::BTreeMap;
use url::Url;

use crate::nix::{self, FlakeRef, JsonMetadata};

#[derive(Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub locks: Locks,
    pub resolved_url: Url,
}

#[derive(Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeName(pub String);

pub type NodeMap = BTreeMap<NodeName, Node>;

#[derive(Clone, Deserialize, PartialEq)]
pub struct Locks {
    pub nodes: NodeMap,
    pub root: NodeName,
}

#[derive(Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct InputName(pub String);

#[derive(Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum InputRef {
    NodeRef(NodeName),
    Follows(Vec<NodeName>),
}

type InputMap = BTreeMap<InputName, InputRef>;

#[derive(Clone, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    pub inputs: Option<InputMap>,
}

impl TryFrom<JsonMetadata> for Metadata {
    type Error = serde_json::Error;

    fn try_from(value: JsonMetadata) -> Result<Self, Self::Error> {
        serde_json::from_str(&value.0)
    }
}

impl TryFrom<&FlakeRef> for Metadata {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: &FlakeRef) -> Result<Self, Self::Error> {
        let json = nix::nix_flake_metadata(value)?;
        json.try_into().wrap_err("JSON deserialization failed")
    }
}

impl TryFrom<FlakeRef> for Metadata {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: FlakeRef) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl Metadata {
    fn root_node(&self) -> Node {
        self.locks
            .nodes
            .get(&self.locks.root)
            .expect("metadata locks should have root node")
            .clone()
    }

    pub fn root_inputs(&self) -> Vec<(InputName, Node)> {
        let nodes = self.nodes();
        if let Some(input_map) = self.root_node().inputs {
            input_map
                .iter()
                .map(|(input_name, input_ref)| {
                    if let InputRef::NodeRef(node_name) = input_ref {
                        let node = nodes
                            .get_key_value(node_name)
                            .expect("metadata should have node referenced by root inputs");
                        (input_name.clone(), node.1.clone())
                    } else {
                        panic!("expected root node inputs to never follow");
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub fn nodes(&self) -> NodeMap {
        self.locks.nodes.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{metadata::Metadata, nix::FlakeRef};

    #[test]
    fn test_finds_root_node_name() {
        let flake_ref = FlakeRef("./test_data".into());
        let metadata: Metadata = flake_ref.try_into().unwrap();
        const EXPECTED_ROOT_NODE_NAME: &str = "root";

        assert_eq!(metadata.locks.root.0, EXPECTED_ROOT_NODE_NAME);
    }

    #[test]
    fn test_includes_all_nodes() {
        let flake_ref = FlakeRef("./test_data".into());
        let metadata: Metadata = flake_ref.try_into().unwrap();
        const EXPECTED_NODE_COUNT: usize = 54;

        let node_count = metadata.locks.nodes.len();

        assert_eq!(node_count, EXPECTED_NODE_COUNT);
    }
}
