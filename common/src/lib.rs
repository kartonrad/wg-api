
pub mod auth;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize)]
pub struct Upload {
    pub id: i32, 
    pub extension: String, 
    pub size_kb: i32, 
    pub original_filename: String
}

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[derive(Serialize, Deserialize)]
pub struct DBUpload {
    id: Option<i32>, 
    extension: Option<String>, 
    original_filename: Option<String>,
    size_kb: Option<i32>
}
// PAIN
impl Into<Option<Upload>> for DBUpload {
    fn into(self) -> Option<Upload> {
        let DBUpload { id, extension, original_filename, size_kb } = self;
        match (id,extension,original_filename,size_kb) {
            (Some(id), Some(extension), Some(original_filename), Some(size_kb)) => {
                Some(Upload { id, extension, original_filename, size_kb })
            },
            _ => None
        }
    }
}
impl From<Option<Upload>> for DBUpload {
    fn from(opt : Option<Upload>) -> Self {
        match opt {
            Some(upl) => DBUpload { id: Some(upl.id), extension: Some(upl.extension), original_filename: Some(upl.original_filename), size_kb: Some(upl.size_kb) },
            None => DBUpload { id: None, extension: None, original_filename: None, size_kb: None }
        }
    }
}



#[derive(Serialize, Deserialize)]
pub struct WG {
    pub id : i32,
    pub url: String,

    pub name: String,
    pub description: String,

    pub profile_pic: Option<DBUpload>,
    pub header_pic: Option<DBUpload>
}