use serde::Serialize;

pub const GRAPH_NAME: &str = "/poolswh/softwareheritage/graph/2024-08-23/compressed/graph";
pub const GRAPH_NAME_TEASER: &str = "/home/infres/rapaport/datasets/2024-08-23-popular-500-python/compressed/graph";

#[derive(Serialize, Debug, Clone)]
pub struct Changes{
    pub commit: String,
    pub created: String,
    pub deleted: String,
}