#[macro_use] extern crate serde_derive;

use std::collections::HashMap;

pub type Timestamp = i64;
pub type KeystrokesStats = HashMap<Timestamp, u32>;

#[derive(Serialize, Deserialize)]
pub struct SynchronizeRequest {
    pub staging_area : KeystrokesStats,
    pub today : Timestamp,
}

#[derive(Serialize, Deserialize)]
pub struct Summary {
    pub oldest_timestamp : Timestamp,
    pub global_count : u64,
    pub today_timestamp : Timestamp,
    pub today_count : u64,
}
