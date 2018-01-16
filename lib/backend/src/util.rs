// TODO change this to return a type-safe thing rather than just 'String'
// pub fn hash_file(absolute_directory: String) -> Result<String, Error> {
//     debug!("Hashing file: {}", absolute_directory);
//     match File::open(&absolute_directory) {
//         Ok(mut file) => {
//             let mut vec_file = &mut Vec::new();
//             let _ = file.read_to_end(&mut vec_file);
//             let sha = digest::digest(&digest::SHA256, &vec_file);

//             // HACK: needs a fix of some description
//             Ok(format!("{:?}", sha).to_lowercase())
//         }
//         Err(e) => Err(e.into()),
//     }
// }
