use serde::{Deserialize, Serialize};

pub const GRAPH_NAME: &str = "/poolswh/softwareheritage/graph/2025-05-18/compressed/graph";
pub const GRAPH_NAME_TEASER: &str = "/home/infres/rapaport/datasets/2024-08-23-popular-500-python/compressed/graph";
pub const ORC_BATCH_SIZE: usize = 1024;

pub const AMOUNT_MERGE_TEASER: usize = 1_123_432;

pub struct Options{
    pub graph: String,
    pub results: String,
    pub amount_merge: Option<usize>
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Changes{
    pub commit: String,
    pub created: String,
    pub deleted: String,
    pub poisoned: bool,
    pub message: String,
    pub message_status: MsgStatus,
    // add status message: {utf8, utf16, can't read}
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub enum MsgStatus{
    Utf8,
    Unreadable,
}