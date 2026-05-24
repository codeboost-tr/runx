//! Packet index contract (`runx.packet.index.v1`): the manifest of materialized
//! skill packets and their content hashes.
use serde::{Deserialize, Serialize};

use crate::schema::RunxSchema;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
pub enum PacketIndexSchema {
    #[serde(rename = "runx.packet.index.v1")]
    V1,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
pub struct PacketIndexEntry {
    pub id: String,
    pub package: String,
    pub version: String,
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, RunxSchema)]
#[serde(deny_unknown_fields)]
#[runx_schema(id = "runx.packet.index.v1")]
pub struct PacketIndex {
    pub schema: PacketIndexSchema,
    pub packets: Vec<PacketIndexEntry>,
}
