//! Extron node metadata uses the exporter's private `exui` key and exui-node/1 identity fields.

use figcraft_doc::Node;
use serde::{Deserialize, Serialize};

use crate::model::{Identity, StateRole};

pub const KEY: &str = "exui";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Page,
    Button,
    Slider,
    Label,
    Shape,
    State,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub schema: String,
    pub owner_node_id: String,
    pub kind: Kind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_id: Option<u16>,
    // Passive element assignments explicitly carry a null controlId.
    pub control_id: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<StateRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u16>,
}

impl Metadata {
    pub fn new(node: &Node, kind: Kind) -> Self {
        // In the exporter schema slider and shape are v2 extensions of the v1 identity.
        let schema = if matches!(kind, Kind::Slider | Kind::Shape) { "exui-node/2" } else { "exui-node/1" };
        Self {
            schema: schema.into(),
            owner_node_id: node.id.to_string(),
            kind,
            key: None,
            object_id: None,
            control_id: None,
            profile: None,
            project_name: None,
            role: None,
            state_id: None,
            state_name: None,
            min: None,
            max: None,
        }
    }

    pub fn read(node: &Node) -> Result<Self, String> {
        let raw = node.plugin_data.get(KEY).ok_or("Missing Extron assignment.")?;
        let data: Self = serde_json::from_str(raw).map_err(|err| format!("Invalid Extron assignment: {err}"))?;
        if !matches!(data.schema.as_str(), "exui-node/1" | "exui-node/2") {
            return Err(format!("Unsupported Extron assignment schema {}.", data.schema));
        }
        if data.owner_node_id != node.id.to_string() {
            return Err("Copied Extron assignment needs its own key and IDs.".into());
        }
        Ok(data)
    }

    pub fn write(&self, node: &mut Node) -> Result<(), serde_json::Error> {
        node.plugin_data.insert(KEY.into(), serde_json::to_string(self)?);
        Ok(())
    }

    pub fn identity(&self) -> Result<Identity, String> {
        Ok(Identity {
            key: self.key.clone().ok_or("Missing semantic key.")?,
            object_id: self.object_id.ok_or("Missing Object ID.")?,
            control_id: self.control_id,
        })
    }
}
