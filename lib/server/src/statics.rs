use lazy_static::lazy_static;
use prometheus::{IntCounter, IntGauge};

//  Metrics static values executed at runtime and registered to default
//  prometheus registry
lazy_static! {
    pub static ref TOTAL_SPACE: IntGauge = register_int_gauge!(opts!(
        "total_space",
        "available space in bytes in the filesystem containing the data_path",
        labels! {"type" => "disk"}
    )).unwrap();
    pub static ref FREE_SPACE: IntGauge = register_int_gauge!(opts!(
        "free_space",
        "free space in bytes in the filesystem containing the data_path",
        labels! {"type" => "disk"}
    )).unwrap();
    pub static ref AVAILABLE_SPACE: IntGauge = register_int_gauge!(opts!(
        "available_space",
        "available space to non-privileged users in bytes in the filesystem containing the data_path",
        labels! {"type" => "disk"}
    )).unwrap();
    pub static ref TOTAL_MANIFEST_REQUESTS: IntCounter  = register_int_counter!(opts!(
        "total_manifest_requests",
        "total manifests requests made to trow",
        labels! {"type" => "manifests"}
    )).unwrap();
    pub static ref TOTAL_BLOB_REQUESTS: IntCounter  = register_int_counter!(opts!(
        "total_blob_requests",
        "total blobs requests made to trow",
        labels! {"type" => "blobs"}
    )).unwrap();
}