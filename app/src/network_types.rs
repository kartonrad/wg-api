
use common::{auth::IIdentity, WG};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WGMember {
    pub identity : IIdentity,
    pub wg: WG
}