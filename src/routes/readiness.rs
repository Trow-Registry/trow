use std::io;
use std::fs::File;

use rocket::{get};
use rocket_contrib::json::{Json, JsonValue};

pub mod readiness {
    /*
    * Trow readiness endpoint
    * GET /readiness
    */
    fn check_data_dir_perm(data_path: &String) -> io::Result<bool> {
        let file = File::open(data_path)?;
        let metadata = file.metadata()?;
        let permissions = metadata.permissions();
        Ok(permissions.readonly())
    }
    
    #[get("/readiness")]
    pub fn readiness(
        // tc: State<TrowConfig>
    
    ) ->  JsonValue {
        match  check_data_dir_perm(&tc.data_dir) {
            Err(why) => { panic!("{:?}", why) }
            Ok(bool) => {
                if bool {
                    json!({"status": "error" ,"code": "400" ,"success": false, "reason": "data directory is readonly" })
                } else {
                    json!({"status": "ok"})
                }
            }
    
        }
    
    }
}
