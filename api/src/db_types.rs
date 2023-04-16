// Newtype pattern to implement actix_web::error::ResponseError for sqlx::Error (indirectly)
#[derive(Debug)]
pub struct DatabaseError(pub sqlx::Error);

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        DatabaseError(err)
    }
}

// APPROACH 1
pub trait ToDatabase<T> {
    fn handle(self) -> Result<T,DatabaseError>;
}

impl<T> ToDatabase<T> for Result<T, sqlx::Error> {
    fn handle(self) -> Result<T,DatabaseError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => {
                warn!("Database Error Occurred: {}", err);
                Err(err.into())
            }
        }
    }
}

// ================================================================================== Traits + Methods ============================================================================
use async_trait::async_trait;
use common::*;
use crate::{DB_POOL, db};
use rust_decimal::Decimal;

#[async_trait]
pub trait WGExt : Sized { async fn get(id: i32) -> Result<Self, DatabaseError>; async fn get_url(url : &str) -> Result<Self, DatabaseError>; }

#[async_trait]
impl WGExt for WG {
    async fn get(id: i32) -> Result<Self, DatabaseError> {
        sqlx::query_as!(WG, r#"SELECT wgs.id, url, name, description, 
            (pp.id, pp.extension, pp.original_filename, pp.size_kb) as "profile_pic: DBUpload",
            (hp.id, hp.extension, hp.original_filename, hp.size_kb) as "header_pic: DBUpload"
        FROM wgs 
        LEFT JOIN uploads AS pp ON profile_pic = pp.id
        LEFT JOIN uploads AS hp ON header_pic = hp.id
        WHERE wgs.id = $1"#, id)
                .fetch_one(db!()).await.handle()
    }
    
    async fn get_url(url : &str) -> Result<Self, DatabaseError> {
        sqlx::query_as!(WG, r#"SELECT wgs.id, url, name, description, 
            (pp.id, pp.extension, pp.original_filename, pp.size_kb) as "profile_pic: DBUpload",
            (hp.id, hp.extension, hp.original_filename, hp.size_kb) as "header_pic: DBUpload"
        FROM wgs 
        LEFT JOIN uploads AS pp ON profile_pic = pp.id
        LEFT JOIN uploads AS hp ON header_pic = hp.id
        WHERE wgs.url = $1"#, url)
                .fetch_one(db!()).await.handle()
    }
}


#[async_trait]
pub trait UserExt { async fn fetch_all_wg(wg_id: i32) -> Result<Vec<User>, DatabaseError>; }

#[async_trait]
impl UserExt for User {
    async fn fetch_all_wg(wg_id: i32) -> Result<Vec<User>, DatabaseError> {
        sqlx::query_as!(User, r#"SELECT users.id, name, bio, username, 
            (pp.id, pp.extension, pp.original_filename, pp.size_kb) as "profile_pic: DBUpload"
        FROM users 
        LEFT JOIN uploads AS pp ON profile_pic = pp.id
        WHERE users.wg = $1"#, wg_id)
            .fetch_all(db!()).await.handle()
    }
}



#[async_trait]
pub trait CostExt : Sized { async fn get_all_balance(user_id: i32, wg_id: i32, balance_id: i32) -> Result<Vec<Self>, DatabaseError>; }

#[async_trait]
impl CostExt for Cost {
    async fn get_all_balance(user_id: i32, wg_id: i32, balance_id: i32) -> Result<Vec<Self>, DatabaseError> {
        sqlx::query_as!(Cost, r#"
        SELECT costs.id, wg_id, name, amount, creditor_id, equal_balances, (pp.id, pp.extension, pp.original_filename, pp.size_kb) as "receit: DBUpload",
            added_on, ROW(my_share.cost_id, my_share.debtor_id, my_share.paid) as "my_share: DBCostShare",
            count(*) as nr_shares, sum( CASE WHEN shares.paid = false AND shares.debtor_id != creditor_id THEN 1 ELSE 0 END ) as nr_unpaid_shares       
        FROM costs
        LEFT JOIN cost_shares as shares ON costs.id = shares.cost_id -- multiple per row
        LEFT JOIN cost_shares as my_share ON costs.id = my_share.cost_id AND my_share.debtor_id = $1 -- guarranteed to be unique per row, as (cost_id, debtor_id) is PRIMARY
        LEFT JOIN uploads AS pp ON receit_id = pp.id
        WHERE wg_id = $2 AND coalesce(equal_balances, 0) = $3
        GROUP BY costs.id, my_share.cost_id, my_share.debtor_id, my_share.paid, pp.id, pp.extension, pp.original_filename, pp.size_kb
        ORDER BY added_on DESC;"#, user_id, wg_id, balance_id)
            .fetch_all(db!()).await.handle()
    }
}



#[async_trait]
pub trait CostShareExt : Sized {
    async fn get_all_cost(cost_id: i32, wg_id: i32) -> Result<Vec<Self>, DatabaseError>;
}

#[async_trait]
impl CostShareExt for CostShare {
    async fn get_all_cost(cost_id: i32, wg_id: i32) -> Result<Vec<Self>, DatabaseError> {
        sqlx::query_as!(CostShare, "SELECT cost_id, debtor_id, paid 
        FROM cost_shares LEFT JOIN costs ON cost_id = costs.id
        WHERE cost_id=$1 AND costs.wg_id = $2", cost_id, wg_id)
            .fetch_all(db!()).await.handle()
    }
}


#[async_trait]
pub trait UserDebtExt : Sized { async fn get_all_for_balance(wg_id : i32, balance_id: i32) -> Result<Vec<Self>, DatabaseError>; }

#[async_trait]
impl UserDebtExt for UserDebt {
    async fn get_all_for_balance(wg_id : i32, balance_id: i32) -> Result<Vec<Self>, DatabaseError> {
        struct DebtTableRecord {
            u1: Option<i32>, 
            to_recieve: Option<Decimal>, 
            u2: Option<i32>, 
            to_pay: Option<Decimal> 
        }
        
        /*
            OK DAMN!! Let's explain this query.
            First, the subquery in the first section [debt_table > cost_agg] gets a full table of costs, with the number of their shares included on each one.
            (TODO: wg and balance filtering of costs could maybe already occour on this level tbh - even before the join and agg would be ideal)

            This information is used to compute the entire first section [debt_table],
            which returns a big list of back-and-forth payments for every share on every cost - if it's not already payed etc.
            ==> TODO! Even though its a sacrifice in rigidity on the db level, having an amount field on the shares would GREATLY improve READ performance
            ==> for now it's fine, but keep in mind that the entire first section can be dropped in favour of the cost_shares.amount field!!
            ==> the tradeoff would be more than worth it, as this query runs on every visit!!

            These debt_table is then summed up, once for what every creditor recieves, and once for every debtor pays.
            Meaning, that the sections [pay_table] and [recieve_table] differ in their GROUP BY
            Thats why they are mutually exclusive, and require the WITH clause

            finally, they are both joined.
        */
        let dtrs: Vec<DebtTableRecord> = sqlx::query_as!( DebtTableRecord , r#"
            WITH debt_table AS (                                                           
                SELECT debtor_id, creditor_id, (amount/nr_shares)::NUMERIC(16,2) as owed
                FROM cost_shares
                LEFT JOIN (
                    SELECT costs.id, amount, creditor_id, wg_id, equal_balances,
                        count(*) as nr_shares, sum( CASE WHEN shares.paid = false AND shares.debtor_id != creditor_id THEN 1 ELSE 0 END ) as nr_unpaid_shares
                    FROM costs
                    LEFT JOIN cost_shares as shares ON costs.id = shares.cost_id   --multiple per row
                    GROUP BY costs.id
                ) AS cost_agg ON cost_agg.id = cost_shares.cost_id
                WHERE debtor_id != creditor_id AND paid = false AND cost_agg.wg_id = $1 AND coalesce(equal_balances, 0) = $2
            ), recieve_table AS (                                                   
                SELECT creditor_id as user_id, sum(owed) as to_recieve
                FROM debt_table
                GROUP BY creditor_id
            ), pay_table AS (
                SELECT debtor_id as user_id, sum(owed) as to_pay
                FROM debt_table
                GROUP BY debtor_id
            )
            SELECT recieve_table.user_id as u1, to_recieve, pay_table.user_id as u2, to_pay FROM recieve_table
            FULL OUTER JOIN pay_table ON( recieve_table.user_id = pay_table.user_id );"#
        , wg_id, balance_id)
            .fetch_all(db!()).await.handle()?;

        // this is important logic! and different from the conversion between the usual DB* types
        // u1 and to_recieve may be None, while u2 and to_pay are Some (and vice versa)
        let mut debts: Vec<UserDebt> = vec![];
        for record in dtrs {
            let user_id = record.u1.or(record.u2);
            if let Some (user_id) = user_id {
                debts.push(UserDebt {
                    user_id,
                    to_recieve: record.to_recieve.unwrap_or(Decimal::ZERO),
                    to_pay:  record.to_pay.unwrap_or(Decimal::ZERO)
                })
            }
        }

        Ok(debts)
    }
}