use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub type Timestamp = i64;
pub type StagingArea = HashMap<Timestamp, u32>;

#[derive(Serialize, Deserialize)]
pub struct SynchronizeRequest {
    pub staging_area : StagingArea,
    pub today : Timestamp,
}

#[derive(Serialize, Deserialize)]
pub struct Summary {
    pub oldest_timestamp : Timestamp,
    pub global_count : u64,
    pub today_timestamp : Timestamp,
    pub today_count : u64,
}
