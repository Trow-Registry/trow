use std::path::Path;

use anyhow::Result;
use lazy_static::lazy_static;
use prometheus::{
    labels, opts, register_int_counter, register_int_gauge, Encoder, IntCounter, IntGauge,
    TextEncoder,
};

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
        "total number of requests for manifests made",
        labels! {"type" => "manifests"}
    )).unwrap();
    pub static ref TOTAL_BLOB_REQUESTS: IntCounter  = register_int_counter!(opts!(
        "total_blob_requests",
        "total number of requests for blobs made",
        labels! {"type" => "blobs"}
    )).unwrap();
}

// Query disk metrics
pub fn query_disk_metrics(path: &Path) {
    let data_path = path.parent().unwrap();
    let available_space = fs3::available_space(data_path).unwrap_or(0);
    AVAILABLE_SPACE.set(available_space as i64);
    let free_space = fs3::free_space(data_path).unwrap_or(0);
    FREE_SPACE.set(free_space as i64);
    let total_space = fs3::total_space(data_path).unwrap_or(0);
    TOTAL_SPACE.set(total_space as i64);
}

pub fn gather_metrics(blobs_path: &Path) -> Result<String> {
    query_disk_metrics(blobs_path);

    let encoder = TextEncoder::new();

    // Gather all prometheus metrics from the DEFAULT_REGISTRY
    //      * disk
    //      * total manifest requests
    //      * total blob requests

    let metric_families = prometheus::gather();
    let mut buffer = vec![];

    encoder.encode(&metric_families, &mut buffer)?;

    let metrics = String::from_utf8(buffer)?;

    Ok(metrics)
}
