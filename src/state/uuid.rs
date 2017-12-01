use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;

use failure::Error;
use ring::digest;

pub struct UuidImpl {
    digests: HashSet<String>,
}
impl UuidImpl {
    pub fn new() -> UuidImpl {
        UuidImpl { digests: HashSet::<String>::new() }
    }
}

pub fn scratch_path(uuid: &String) -> String {
    warn!("Deprecated, please use the recommended function");
    format!("data/scratch/{}", uuid)
}

pub fn layer_path(fname: &String) -> String {
    warn!("Deprecated, please use the recommended function");
    format!("data/layers/{}", fname)
}

/// given a _uuid_ and a _hash_, will copy the layer to the _layers_
/// directory from the _scratch_ directory.
pub fn save_layer(uuid: &String, digest: &String) -> io::Result<u64> {
    let from = scratch_path(uuid);
    let to = layer_path(digest);

    // TODO: check if layer already exists.
    debug!("Copying {} -> {}", from, to);
    fs::copy(from, to)
}

/// Marks the given uuid for deletion.
/// The current implementation simply deletes the file, future
/// implementations may want to propogate the message to neighbouring
/// registry instances.
pub fn mark_delete(uuid: &String) -> io::Result<()> {
    let file = scratch_path(uuid);
    fs::remove_file(file)
}


// TODO change this to return a type-safe thing rather than just 'String'
pub fn hash_file(absolute_directory: String) -> Result<String, Error> {
    debug!("Hashing file: {}", absolute_directory);
    match File::open(&absolute_directory) {
        Ok(mut file) => {
            let mut vec_file = &mut Vec::new();
            let _ = file.read_to_end(&mut vec_file);
            let sha = digest::digest(&digest::SHA256, &vec_file);

            // HACK: needs a fix of some description
            Ok(format!("{:?}", sha).to_lowercase())
        }
        Err(e) => Err(e.into()),
    }
}
