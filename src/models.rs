use serde::Serialize;

#[derive(Serialize)]
pub struct RucInfo {
    pub ruc: String,
    pub dv: String,
    pub name: String,
    pub status: String,
}
