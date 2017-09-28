use std::fs::File;
use std::io::Read;

use ring::digest;
use uuid::Uuid;

#[derive_FromForm]
#[derive(Debug)]
pub struct DigestStruct {
    pub query: bool,
    pub digest: String,
}

// TODO change this to return a type-safe thing rather than just 'String'
pub fn scratch_path(uuid: &String) -> String {
    format!("data/scratch/{}", uuid)
}

// TODO change this to return a type-safe thing rather than just 'String'
pub fn hash_file(absolute_directory: String) -> Result<String, String> {
    debug!("Hashing file: {}", absolute_directory);
    match File::open(&absolute_directory) {
        Ok(mut file) => {
            let mut vec_file = &mut Vec::new();
            let _ = file.read_to_end(&mut vec_file);
            let sha = digest::digest(&digest::SHA256, &vec_file);

            // HACK: needs a fix of some description
            Ok(format!("{:?}", sha).to_lowercase())
        }
        Err(_) => Err(format!("could not open file: {}", absolute_directory))
    }
}


pub fn gen_uuid() -> Uuid {
    Uuid::new_v4()
}
