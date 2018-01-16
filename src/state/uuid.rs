pub fn scratch_path(uuid: &String) -> String {
    warn!("Deprecated, please use the recommended function");
    format!("data/scratch/{}", uuid)
}
