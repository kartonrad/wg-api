
use std::collections::HashMap;

use common::{auth::IIdentity, WG, DBUpload, Upload, User};
use serde::{Serialize, Deserialize};

pub type HTTP = reqwest::Client;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WGMember {
    pub identity : IIdentity,
    pub wg: WG,
    pub friends: HashMap<i32, User>
}

pub fn get_upload( opt_upload: Option<DBUpload> ) -> Option<String> {
    if let Some(header) = opt_upload {
        let header : Option<Upload> = header.into();
        if let Some(header) = header {
            return Some(header.into_url());
        }
    }
    
    None
}