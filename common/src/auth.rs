
use time::{ext::NumericalDuration};

use serde::{Serialize, Deserialize};
use time::PrimitiveDateTime;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(into = "SerdeIdentity")] 
pub struct IIdentity {
    pub id: i32,
    pub profile_pic: Option<i32>,
    pub name: String,
    pub bio: String,

    pub username: String, // '[abcdefghijklmopqrstuvwxyz0123456789-_]+'
    pub password_hash: String,
    pub revoke_before: time::PrimitiveDateTime,

    pub wg: Option<i32>
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct SerdeIdentity {
    pub id: i32,
    pub profile_pic: Option<i32>,
    pub name: String,
    pub bio: String,
    pub username: String,
    pub password_hash: String,
    pub wg: Option<i32>,
    #[serde(with = "time::serde::rfc3339")]
    pub revoke_before: time::OffsetDateTime
}
impl Into<SerdeIdentity> for IIdentity {
    fn into(self) -> SerdeIdentity {
        SerdeIdentity{ id: self.id, profile_pic: self.profile_pic, name: self.name, bio: self.bio, password_hash: self.password_hash, wg: self.wg, username:self.username,
            revoke_before: self.revoke_before.assume_utc() 
        }
    }
}
impl Into<IIdentity> for SerdeIdentity {
    fn into(self) -> IIdentity {
        IIdentity{ id: self.id, profile_pic: self.profile_pic, name: self.name, bio: self.bio, password_hash: self.password_hash, wg: self.wg, username:self.username,
            revoke_before: PrimitiveDateTime::new(self.revoke_before.date(), self.revoke_before.time()).saturating_add( (self.revoke_before.offset().whole_minutes() as i64).minutes())  
        }
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct LoginInfo {
    pub username: String,
    pub password: String
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Pwd {
    pub password: String
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Token {
    pub token : String,
    pub expires: u64
}