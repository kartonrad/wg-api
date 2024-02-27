use std::{fmt::Display};

use actix_multipart::Multipart;
use actix_web::{ HttpResponse, Responder, get, put, post, http::StatusCode, web, Error, delete,};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use time::OffsetDateTime;

use super::auth::{WGMemberIdentity, Identity};
use crate::{DB_POOL, change_upload, db};
use crate::file_uploads::{multipart_parse, TempUpload, delete_unreferenced_upload};
use common::*;

// ================================================================================== ERROR Handling ==================================================================================

use crate::db_types::*;

// Trait
impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            sqlx::Error::Database(err) => {
                if err.constraint().is_some() {
                    write!(f, "Duplicate Entry/Name may already exist kinda error")
                } else {
                    write!(f, "Internal Server Error")
                }
            },
            sqlx::Error::RowNotFound => write!(f, "Record not found"),
            _ =>  write!(f, "Internal Server Error"),
        }
    }
}

impl actix_web::error::ResponseError for DatabaseError {
    fn status_code(&self) -> StatusCode {
        match &self.0 {
            sqlx::Error::Database(err) => if err.constraint().is_some() {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            } ,
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// ================================================================================== STRUCTS ==================================================================================

#[derive(Serialize, Deserialize)]
struct BalanceInput {
    balance: Option<i32>
}


// ================================================================================== ROUTES ==================================================================================
#[get("/me")]
async fn get_user_me(mut identity: Identity) -> impl Responder {
    identity.password_hash = "<Not Provided>".to_string();

    HttpResponse::Ok()
        .json(identity.0)
}

#[put("/me")]
async fn put_user_me(identity: Identity, payload: Multipart) -> Result<impl Responder, Error> {

    #[derive(Serialize, Default)]
    struct ResJson {
        name: Option<String>,
        bio: Option<String>,
        username: Option<String>,
        profile_pic: Option<Upload>
    }
    let mut res_json = ResJson {
        ..Default::default()
    };

    // Get Multipart Fields
    let mut lmaobozo = multipart_parse(payload, ["name", "bio", "username"], ["profile_pic"]).await?;
    trace!("Bozo fields: {:?}", lmaobozo);

    if let Some(profile_picf) = &mut lmaobozo.1[0] {
        let new_upl = change_upload!("users", "profile_pic", i32)(profile_picf.move_responsibility(), identity.id, None).await;
        if let Ok (new_upl) = new_upl  {
            res_json.profile_pic = Some(new_upl);
        } else {
            warn!("Couldn't change upload :(");
        }
    }
    if let Some(name) = &lmaobozo.0[0] {
        let res = sqlx::query!("UPDATE users SET name=$1 WHERE id=$2", name, identity.id)
            .execute(db!()).await;
        if let Ok(_res) = res {
            res_json.name = Some(name.to_owned());
        }
    }
    if let Some(bio) = &lmaobozo.0[1] {
        let res = sqlx::query!("UPDATE users SET bio=$1 WHERE id=$2", bio, identity.id)
            .execute(db!()).await;
        if let Ok(_res) = res {
            res_json.bio = Some(bio.to_owned());
        }
    }
    if let Some(username) = &lmaobozo.0[2] {
        let res = sqlx::query!("UPDATE users SET username=$1 WHERE id=$2", username, identity.id)
            .execute(db!()).await;
        if let Ok(_res) = res {
            res_json.username = Some(username.to_owned());
        }
    }
    
    Ok(HttpResponse::Ok()
        .json(res_json))
}

// user_change_password, user_revoke_tokens

#[get("/my_wg")]
async fn get_wg(member: WGMemberIdentity) -> Result<impl Responder, DatabaseError> {
    let wgopt = WG::get(member.wg_id).await?;
    
    Ok( HttpResponse::Ok()
    .json(wgopt) )
}

#[get("/wg/{url}")]
async fn get_wg_public(params: web::Path<(String,)>) -> Result<impl Responder, DatabaseError> {
    //let wg = sqlx::query_as!(WG, "SELECT * FROM wgs WHERE id = $1", wg_id)
    let wg = WG::get_url(&(*params).0).await?;

    Ok( HttpResponse::Ok()
    .json(wg) )
}

#[put("/my_wg")]
async fn put_wg(WGMemberIdentity{wg_id, ..} : WGMemberIdentity, payload: Multipart) -> Result<impl Responder, Error> {

    #[derive(Serialize, Default)]
    struct ResJson {
        name: Option<String>,
        url: Option<String>,
        description: Option<String>,
        profile_pic: Option<Upload>,
        header_pic: Option<Upload>
    }
    let mut res_json = ResJson {
        ..Default::default()
    };

    // Get Multipart Fields
    let mut lmaobozo = multipart_parse(payload, ["name", "url", "description"], ["profile_pic", "header_pic"]).await?;
    trace!("Bozo fields: {:?}", lmaobozo);

    if let Some(profile_picf) = &mut lmaobozo.1[0] {
        let new_upl = change_upload!("wgs", "profile_pic", i32)(profile_picf.move_responsibility(), wg_id, None).await;
        if let Ok (new_upl) = new_upl  {
            res_json.profile_pic = Some(new_upl);
        } else {
            warn!("Couldn't change upload :(");
        }
    }
    if let Some(header_picf) = &mut lmaobozo.1[1] {
        let new_upl = change_upload!("wgs", "header_pic", i32)(header_picf.move_responsibility(), wg_id, None).await;
        if let Ok (new_upl) = new_upl  {
            res_json.header_pic = Some(new_upl);
        } else {
            warn!("Couldn't change upload :(");
        }
    }
    if let Some(name) = &lmaobozo.0[0] {
        let res = sqlx::query!("UPDATE wgs SET name=$1 WHERE id=$2", name, wg_id)
            .execute(db!()).await;
        if let Ok(_res) = res {
            res_json.name = Some(name.to_owned());
        }
    }
    if let Some(url) = &lmaobozo.0[1] {
        let res = sqlx::query!("UPDATE wgs SET url=$1 WHERE id=$2", url, wg_id)
            .execute(db!()).await;
        if let Ok(_res) = res {
            res_json.url = Some(url.to_owned());
        }
    }
    if let Some(description) = &lmaobozo.0[2] {
        let res = sqlx::query!("UPDATE wgs SET description=$1 WHERE id=$2", description, wg_id)
            .execute(db!()).await;
        if let Ok(_res) = res {
            res_json.description = Some(description.to_owned());
        }
    }
    
    Ok(HttpResponse::Ok()
        .json(res_json))
}

#[get("/my_wg/users")]
async fn get_wg_users(member: WGMemberIdentity) -> Result<impl Responder, DatabaseError>  {
    let wg = User::fetch_all_wg(member.wg_id).await?;
    
    Ok( HttpResponse::Ok()
    .json(wg) )
}

#[get("/wg/{id}/users")]
async fn get_wg_users_public(params: web::Path<(i32,)>) -> Result<impl Responder, DatabaseError>  {
    let wg = User::fetch_all_wg((*params).0).await?;
    
    Ok( HttpResponse::Ok()
    .json(wg) )
}

#[get("/my_wg/costs")]
async fn get_wg_costs(member: WGMemberIdentity, query: web::Query<BalanceInput>) -> Result<impl Responder, DatabaseError> {
    // TODO MIGRATION STRAT: Dings endpunkt mit enum CostOrTrx rückgabe
    // er fetched von beiden tabellen und mischt die dann hier in die richtige reihenfolge!
    let cost = Cost::get_all_balance(member.identity.id, member.wg_id, query.balance.unwrap_or(0)).await?;

    Ok( HttpResponse::Ok()
    .json(cost) )
}

#[post("/my_wg/costs")]
async fn post_wg_costs(WGMemberIdentity{identity, wg_id} : WGMemberIdentity, new_cost: web::Json<CostInput>) -> Result<impl Responder, DatabaseError> {
    let mut trx = db!().begin().await
        ?;

    let cost_id: i32 = sqlx::query_scalar!("INSERT INTO costs (wg_id, name, amount, creditor_id, added_on) VALUES
    ($1, $2, $3, $4, $5) RETURNING id;", wg_id, new_cost.name, new_cost.amount, identity.id, new_cost.added_on)
        .fetch_one(&mut trx).await?;
   
    let users = sqlx::query_scalar!("SELECT id FROM users WHERE wg = $1", wg_id)
        .fetch_all(&mut trx).await?;
    
    for debtor in new_cost.debtors.iter() {
        if !users.contains( &debtor.0 )  {
            continue;
        }
        let mut paid = debtor.1;
        if identity.id == debtor.0 {
            paid = true;
        }

        sqlx::query_scalar!("INSERT INTO cost_shares (cost_id, debtor_id, paid) VALUES
        ($1, $2, $3);", cost_id, debtor.0, paid)
            .execute(&mut trx).await?;
    }
    trx.commit().await?;

    Ok( HttpResponse::Ok()
        .json(cost_id) )
}

#[get("/my_wg/costs/{id}/detail")]
async fn get_wg_costs_id(member: WGMemberIdentity, params: web::Path<(i32,)>) -> Result<impl Responder, DatabaseError> {
    let cost = Cost::get_id(member.identity.id, member.wg_id, (*params).0).await?;

    Ok( HttpResponse::Ok()
        .json(cost) )
}

#[get("/my_wg/costs/{id}/shares")]
async fn get_wg_costs_id_shares(member: WGMemberIdentity, params: web::Path<(i32,)>) -> Result<impl Responder, DatabaseError> {
    let shares = CostShare::get_all_cost((*params).0, member.wg_id).await?;

    Ok( HttpResponse::Ok()
    .json(shares) )
}

#[put("/my_wg/costs/{id}/receit")]
async fn put_wg_costs_id_receit(identity: Identity, payload: Multipart, params: web::Path<(i32,)>) -> Result<impl Responder, Error> {
    
    let id: i32 = sqlx::query_scalar!("SELECT creditor_id FROM costs WHERE id=$1", params.0).fetch_one(db!()).await.handle()?;
    
    if identity.id != id {
        return Ok(HttpResponse::Forbidden().body("Lmao nah you didn't originally post this"));
    }

    let mut lmaobozo = multipart_parse(payload, [], ["receit"]).await?;
    trace!("Bozo fields: {:?}", lmaobozo);

    if let Some(receitf) = &mut lmaobozo.1[0] {
        let new_upl = change_upload!("costs", "receit_id", i32)(receitf.move_responsibility(), (*params).0, identity.wg).await.handle()?;

        return Ok(HttpResponse::Ok().body( format!("Successfully changed receit\n{:?}", new_upl) ));
    }

    Ok(HttpResponse::BadRequest().body("Please provide a 'receit' field using multipart form data"))
}


#[delete("/my_wg/costs/{id}")]
async fn delete_wg_costs_id(identity: Identity, params: web::Path<(i32,)>) -> Result<impl Responder, DatabaseError> {

    let mut trx = db!().begin().await?;


    let cost = sqlx::query!( "SELECT creditor_id, receit_id FROM costs WHERE id=$1;", params.0).fetch_one(&mut trx).await?;
    
    if identity.id != cost.creditor_id {
        return Ok(HttpResponse::Forbidden().body("Lmao nah you didn't originally post this"));
    }

    let formerupload = 
    if let Some(formerupload_id) = cost.receit_id {
        Some ( 
            sqlx::query!("SELECT id, extension, original_filename, size_kb FROM uploads WHERE id=$1",formerupload_id)
            .fetch_one(&mut trx).await? 
        )
    } else {
        None
    };
    
    let _query_res = sqlx::query!("DELETE FROM costs WHERE id = $1", params.0).execute(&mut trx).await?;
    
    trx.commit().await?;

    if let Some(f) = formerupload {
        delete_unreferenced_upload(f.id, db!()).await?;
    }

    return Ok(HttpResponse::Ok().body("success"));
}

#[get("/my_wg/costs/stats")]
async fn get_wg_costs_stats(member: WGMemberIdentity, query: web::Query<BalanceInput>) -> Result<impl Responder, DatabaseError> {
    let debts = UserDebt::get_all_for_balance(member.wg_id, query.balance.unwrap_or(0)).await?;

    Ok( HttpResponse::Ok()
    .json(debts) )
}

#[post("/my_wg/costs/balance")]
async fn post_wg_costs_balance(WGMemberIdentity{identity, wg_id} : WGMemberIdentity) -> Result<impl Responder, DatabaseError> {
    let mut trx = db!().begin().await?;

    let id: i32 = sqlx::query_scalar!("INSERT INTO equal_balances (balanced_on, initiator_id, wg_id) VALUES ('NOW', $1, $2) RETURNING id", identity.id, wg_id).fetch_one(&mut trx).await?;

    let result = sqlx::query!("UPDATE costs SET equal_balances=$1 WHERE equal_balances IS NULL AND wg_id=$2;", id, wg_id).execute(&mut trx).await?;
    
    trx.commit().await?;

    return Ok(HttpResponse::Ok().body(result.rows_affected().to_string()));
}


#[get("/my_wg/costs/balance")]
async fn get_wg_costs_balance(identity: Identity) -> Result<impl Responder, DatabaseError> {
    let costs_opt =
    if let Some(wg_id)  = identity.wg {
        /// TODO Migration Strat: rename to "i_paid_cost, i_recieved_cost" and add "i_paid_trx, i_recieved_trx"
        /// This will make sure that info is there

        let balances = sqlx::query_as!(Balance, r#"
        SELECT equal_balances.id, equal_balances.balanced_on, equal_balances.initiator_id, equal_balances.wg_id, 
            coalesce( sum(costs.amount), 0) as total_unified_spending,
            coalesce( sum( CASE WHEN costs.paid = false AND costs.debtor_id != costs.creditor_id THEN (costs.amount/costs.nr_shares)::NUMERIC(16,2) ELSE 0 END ), 0) as i_paid,
            coalesce( sum( CASE WHEN creditor_id = $2 THEN (costs.amount/costs.nr_shares*costs.nr_unpaid_shares)::NUMERIC(16,2) ELSE 0 END ), 0) as i_recieved,
            coalesce( sum( CASE WHEN costs.paid IS NOT NULL THEN (costs.amount/costs.nr_shares)::NUMERIC(16,2) ELSE 0 END ), 0) AS my_total_spending
        FROM equal_balances
        LEFT JOIN (
            SELECT id, amount, creditor_id, added_on, equal_balances, my_share.paid, my_share.debtor_id,
                count(*) as nr_shares, coalesce( sum( CASE WHEN shares.paid = false AND shares.debtor_id != creditor_id THEN 1 ELSE 0 END) , 0) as nr_unpaid_shares
            FROM costs
            LEFT JOIN cost_shares as shares ON costs.id = shares.cost_id -- multiple per row
            LEFT JOIN cost_shares as my_share ON costs.id = my_share.cost_id AND my_share.debtor_id = $2 -- guarranteed to be unique per row, as (cost_id, debtor_id) is PRIMARY
            WHERE wg_id = $1
            GROUP BY costs.id, my_share.cost_id, my_share.paid, my_share.debtor_id
        ) AS costs ON costs.equal_balances = equal_balances.id
        WHERE wg_id = $1
        GROUP BY equal_balances.id, equal_balances.balanced_on, equal_balances.initiator_id, equal_balances.wg_id
        ORDER BY equal_balances.balanced_on DESC;"#,  wg_id, identity.id)
            .fetch_all(db!()).await?;

        Some(balances)
    } else {
        None
    };

    Ok( HttpResponse::Ok()
    .json(costs_opt) )
}

#[get("/my_wg/costs/over_time/{interval}")]
async fn get_wg_costs_over_time(member: WGMemberIdentity, params: web::Path<(String,)> ) -> Result<impl Responder, DatabaseError> {
    #[derive(Serialize, Deserialize)]
    struct DBRegularSpending {
        time_bucket: Option<OffsetDateTime>,
        total_unified_spending: Option<Decimal>,
        i_paid: Option<Decimal>,
        i_recieved: Option<Decimal>,
        my_total_spending: Option<Decimal>
    }

    // needs no modification for trx
    let balances = sqlx::query_as!(DBRegularSpending, r#"
    SELECT
        date_trunc($3, added_on) as time_bucket ,
        coalesce( sum(costs.amount), 0) as total_unified_spending,
        coalesce( sum( CASE WHEN costs.paid = false AND costs.debtor_id != costs.creditor_id THEN (costs.amount/costs.nr_shares)::NUMERIC(16,2) ELSE 0 END ), 0) as i_paid,
        coalesce( sum( CASE WHEN creditor_id = $2 THEN (costs.amount/costs.nr_shares*costs.nr_unpaid_shares)::NUMERIC(16,2) ELSE 0 END ), 0) as i_recieved,
        coalesce( sum( CASE WHEN costs.paid IS NOT NULL THEN (costs.amount/costs.nr_shares)::NUMERIC(16,2) ELSE 0 END ), 0) AS my_total_spending
    FROM (
        SELECT id, amount, creditor_id, added_on, equal_balances, my_share.paid, my_share.debtor_id,
            count(*) as nr_shares, coalesce( sum( CASE WHEN shares.paid = false AND shares.debtor_id != creditor_id THEN 1 ELSE 0 END) , 0) as nr_unpaid_shares
        FROM costs
        LEFT JOIN cost_shares as shares ON costs.id = shares.cost_id -- multiple per row
        LEFT JOIN cost_shares as my_share ON costs.id = my_share.cost_id AND my_share.debtor_id = $2 -- guarranteed to be unique per row, as (cost_id, debtor_id) is PRIMARY
        WHERE wg_id = $1
        GROUP BY costs.id, my_share.cost_id, my_share.paid, my_share.debtor_id
    ) AS costs
    GROUP BY time_bucket ORDER BY time_bucket DESC;"#, member.wg_id, member.identity.id, params.0)
        .fetch_all(db!()).await?;

    let balances_clean: Vec<RegularSpending> = balances.iter().filter_map( |item| {
        //let lmao = item.i_paid.zip(item.i_recieved).zip(item.my_total_spending).zip(item.total_unified_spending).zip(item.time_bucket);
        if let Some(time) = item.time_bucket {
            //let time = zipped.1;

            let cleaned = RegularSpending {
                i_paid: item.i_paid.unwrap_or(Decimal::from(0)), //zipped.0.0.0.0,
                i_recieved: item.i_recieved.unwrap_or(Decimal::from(0)),//zipped.0.0.0.1,
                my_total_spending: item.my_total_spending.unwrap_or(Decimal::from(0)),//zipped.0.0.1,
                total_unified_spending: item.total_unified_spending.unwrap_or(Decimal::from(0)), //zipped.0.1,
                time_bucket: time,
                week: time.iso_week(),
                month: time.month() as u8,
                year: time.year()
            };
            Some(cleaned)
        } else {
            None
        }
    }).collect();

    Ok( HttpResponse::Ok()
    .json(balances_clean) )
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            //.wrap(Authentication)
            .service(get_user_me)
            .service(put_user_me)
            .service(get_wg)
            .service(get_wg_public)
            .service(put_wg)
            .service(get_wg_users)
            .service(get_wg_users_public)
            .service(get_wg_costs)
            .service(post_wg_costs)
            .service(get_wg_costs_id)
            .service(get_wg_costs_stats)

            .service(get_wg_costs_id_shares)
            .service(put_wg_costs_id_receit)
            .service(delete_wg_costs_id)

            .service(post_wg_costs_balance)
            .service(get_wg_costs_balance)

            .service(get_wg_costs_over_time)
    );  
}

/*
 let filepath=format!("uploads/temp/{}{}", temp_upload.local_id, match get_mime_extensions(field.content_type()) {
            Some(ext) => {
                let mut str = ".".to_string();
                str.push_str( ext.first().map(|s| *s).unwrap_or("cringe") );
                str
            },
            None=>"".to_string()
        } );
 */