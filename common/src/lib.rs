
pub mod auth;

use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;
use time::OffsetDateTime;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Upload {
    pub id: i32, 
    pub extension: String, 
    pub size_kb: i32, 
    pub original_filename: String
}

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct DBUpload {
    pub id: Option<i32>, 
    pub extension: Option<String>, 
    pub original_filename: Option<String>,
    pub size_kb: Option<i32>
}



#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct WG {
    pub id : i32,
    pub url: String,

    pub name: String,
    pub description: String,

    pub profile_pic: Option<DBUpload>,
    pub header_pic: Option<DBUpload>
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id : i32,
    pub username: String,

    pub name: String,
    pub bio: String,

    pub profile_pic: Option<DBUpload>,
}


#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct CostShare {
    pub cost_id: i32, 
    pub debtor_id: i32,
    pub paid: bool
}

#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct DBCostShare {
    pub cost_id: Option<i32>, 
    pub debtor_id: Option<i32>,
    pub paid: Option<bool>
}

/// Struct passed to the backend, to create a cost and it's shares.
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct CostInput {
    pub name: String,
    pub amount: rust_decimal::Decimal,
    #[serde(with= "time::serde::iso8601")]
    pub added_on: time::OffsetDateTime,
    pub debtors: Vec<(i32, bool)>
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Cost {
    pub id: i32,
    pub wg_id : i32,
    pub name: String,
    pub amount: rust_decimal::Decimal,
    pub creditor_id: i32,
    #[serde(with= "time::serde::rfc3339")]
    pub added_on: time::OffsetDateTime,
    pub equal_balances: Option<i32>,

    pub receit: Option<DBUpload>,
    pub my_share: Option<DBCostShare>,
    pub nr_shares: Option<i64>,
    pub nr_unpaid_shares:  Option<i64>
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct UserDebt {
    pub user_id: i32,
    pub to_recieve: Decimal,
    pub to_pay: Decimal
}


#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Balance {
    pub id: i32,
    #[serde(with= "time::serde::rfc3339")]
    pub balanced_on: OffsetDateTime,
    pub initiator_id: i32,
    pub wg_id: i32,
    pub total_unified_spending: Option<Decimal>,
    pub i_paid: Option<Decimal>,
    pub i_recieved: Option<Decimal>,
    pub my_total_spending: Option<Decimal>
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct RegularSpending {
    #[serde(with= "time::serde::rfc3339")]
    pub time_bucket: OffsetDateTime,
    pub year: i32, pub week: u8, pub month: u8,
    pub total_unified_spending: Decimal,
    pub i_paid: Decimal,
    pub i_recieved: Decimal,
    pub my_total_spending: Decimal
}


// ======== CONVERT BETWEEN THE ALL_FIELD NULLABLE DB TYPES RETURNED FROM SQLX AND THE PURE VERSION
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
// PAIN
impl Into<Option<CostShare>> for DBCostShare {
    fn into(self) -> Option<CostShare> {
        let DBCostShare { cost_id, debtor_id, paid } = self;
        match (cost_id, debtor_id, paid) {
            (Some(cost_id), Some(debtor_id), Some(paid)) => {
                Some(CostShare { cost_id, debtor_id, paid })
            },
            _ => None
        }
    }
}
impl From<Option<CostShare>> for DBCostShare {
    fn from(opt : Option<CostShare>) -> Self {
        match opt {
            Some(upl) => DBCostShare { cost_id: Some(upl.cost_id), debtor_id: Some(upl.debtor_id), paid: Some(upl.paid) },
            None => DBCostShare { cost_id: None, debtor_id: None, paid: None }
        }
    }
}
