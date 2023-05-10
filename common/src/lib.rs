
pub mod auth;

use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

use serde::{Serialize, Deserialize};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use time::OffsetDateTime;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Upload {
    pub id: i32, 
    pub extension: String, 
    pub size_kb: i32, 
    pub original_filename: String
}
impl Upload {
    pub fn into_url(self) -> String {
        format!("/uploads/{}.{}", self.id, self.extension)
    }
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
    pub amount: Decimal,
    #[serde(with= "time::serde::iso8601")]
    pub added_on: OffsetDateTime,
    pub debtors: Vec<(i32, bool)>
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Cost {
    pub id: i32,
    pub wg_id : i32,
    pub name: String,
    pub amount: Decimal,
    pub creditor_id: i32,
    #[serde(with= "time::serde::rfc3339")]
    pub added_on: OffsetDateTime,
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
impl Into<UserNetDebt> for UserDebt {
    fn into(self) -> UserNetDebt {
        UserNetDebt {
            user_id: self.user_id,
            net_tally: self.to_recieve-self.to_pay
        }
    }
}

#[derive(Debug)]
pub struct UserNetDebt {
    pub user_id: i32,
    pub net_tally: Decimal,
}

impl Eq for UserNetDebt {}

impl PartialEq<Self> for UserNetDebt {
    fn eq(&self, other: &Self) -> bool {
        self.net_tally==other.net_tally
    }
}

impl PartialOrd<Self> for UserNetDebt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UserNetDebt {
    fn cmp(&self, other: &Self) -> Ordering {
        self.net_tally.cmp(&other.net_tally)
    }
}

#[derive(Debug,Clone)]
pub struct BalancingError;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct BalancingTransaction {
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub amt: Decimal
}
impl BalancingTransaction {
    // Assumption: Die müssen equal balancen können:
    // Ansonsten gibts den BalancingError!
    pub fn from_debt_table( table: Vec<UserDebt> ) -> Result<Vec<Self>, BalancingError> {
        let mut trx = Vec::new();

        let mut net_creditors: Vec<UserNetDebt> = Vec::new();
        let mut net_debtors: Vec<UserNetDebt> = Vec::new();

        table.into_iter().for_each(| entry | {
            let net: UserNetDebt = entry.into();

            if net.net_tally.is_sign_positive() && !net.net_tally.is_zero() {
                net_creditors.push(net);
            } else if net.net_tally.is_sign_negative() && !net.net_tally.is_zero() {
                net_debtors.push(net);
            }
            // people with zero are filtered out here -> they cause no transactions
        });

        // sort ist ascending
        net_creditors.sort_unstable(); // das heißt der beginnt mit kleinen positiven zahlen (itarate: ende bis anfang)
        net_debtors.sort_unstable();// und der begint mit den großen negativen zahlen (iterate: anfang bis ende)
        dbg!(&net_debtors, &net_creditors);
        let mut left_slice = &mut net_creditors[..];
        //let mut i = 0;

        for mut debtor in net_debtors.into_iter() {
            while debtor.net_tally < Decimal::from(0u8) {
                //i+=1; if i>10 { panic!("AHHH WAY TOO MANY TRX!"); };
                let creditor = left_slice.last_mut().ok_or(BalancingError)?;
                let mut creditor_done = false;
                let ctally = creditor.net_tally;
                let dtally = -debtor.net_tally;

                let addor =
                if ctally <= dtally {
                    // creditor is paid off
                    creditor_done = true;
                    ctally
                } else {
                    // debtor has paid everything
                    dtally
                };
                dbg!(&creditor, &debtor, ctally, dtally, addor);

                debtor.net_tally += addor;
                creditor.net_tally -= addor;
                assert!(debtor.net_tally <= Decimal::from(0u8) );
                assert!(creditor.net_tally >= Decimal::from(0u8) );

                let ntrx = Self {
                    from_user_id: debtor.user_id,
                    to_user_id: creditor.user_id,
                    amt: addor,
                };
                println!("{:?}", ntrx);
                trx.push(ntrx);

                // down here because of borrow checker rules
                if creditor_done {
                    // remove creditor
                    left_slice = left_slice.split_last_mut().expect("because we checked above that last exists").1;
                }
            }

        }

        Ok(trx)
    }
}

#[test]
fn test_balance() {
    use rust_decimal_macros::dec;

    let res =
        BalancingTransaction::from_debt_table(
            vec![
                UserDebt { user_id: 1, to_pay: dec!(0.50), to_recieve: dec!(15.50) },
                UserDebt { user_id: 2, to_pay: dec!(0.50), to_recieve: dec!(35.50) },
                UserDebt { user_id: 3, to_pay: dec!(5.50), to_recieve: dec!(55.50) },
                UserDebt { user_id: 4, to_pay: dec!(75.50), to_recieve: dec!(0.50) },
                UserDebt { user_id: 5, to_pay: dec!(125.50), to_recieve: dec!(100.50) }
            ]
        ).expect("Creating transactions from debt table to succeed");
    //panic!("{:?}", res);
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
pub enum RegularDef {
    Millennium,
    Century,
    Decade,
    Year,
    Quarter,
    Month,
    Week,
    Day
}
impl Display for RegularDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RegularDef::Millennium => f.write_str("millennium"),
            RegularDef::Century =>  f.write_str("century"),
            RegularDef::Decade =>  f.write_str("decade"),
            RegularDef::Year =>  f.write_str("year"),
            RegularDef::Quarter => f.write_str("quarter"),
            RegularDef::Month =>  f.write_str("month"),
            RegularDef::Week =>  f.write_str("week"),
            RegularDef::Day =>  f.write_str("day"),
        }
    }
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
impl Default for RegularSpending {
    fn default() -> Self {
        Self {
            time_bucket: OffsetDateTime::UNIX_EPOCH,
            year: 0,
            week: 0,
            month: 0,
            total_unified_spending: dec!(0.0),
            i_paid: dec!(0.0),
            i_recieved: dec!(0.0),
            my_total_spending: dec!(0.0),
        }
    }
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
