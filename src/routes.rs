use actix_multipart::Multipart;
use actix_web::{ HttpResponse, Responder, get, put, delete, post, http::StatusCode, dev::{ConnectionInfo}, web, Error,};
use serde::Serialize;
use crate::{DB_POOL, change_upload, auth::res_error, file_uploads::{Upload, DBRetrUpload}};

use super::auth::Identity;

use crate::file_uploads::{multipart_parse, AscendingUpload, TempUpload};

// ================================================================================== ROUTES ==================================================================================
#[get("/me")]
async fn get_user_me(mut identity: Identity) -> impl Responder {
    identity.password_hash = "<Not Provided>".to_string();

    HttpResponse::Ok()
        .json(identity)
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
        let new_upl = change_upload!("users", "profile_pic", i32)(profile_picf.move_responsibility(), identity.id).await;
        if let Ok (new_upl) = new_upl  {
            res_json.profile_pic = Some(new_upl);
        } else {
            warn!("Couldn't change upload :(");
        }
    }
    if let Some(name) = &lmaobozo.0[0] {
        let res = sqlx::query!("UPDATE users SET name=$1 WHERE id=$2", name, identity.id)
            .execute(DB_POOL.get().await).await;
        if let Ok(_res) = res {
            res_json.name = Some(name.to_owned());
        }
    }
    if let Some(bio) = &lmaobozo.0[1] {
        let res = sqlx::query!("UPDATE users SET bio=$1 WHERE id=$2", bio, identity.id)
            .execute(DB_POOL.get().await).await;
        if let Ok(_res) = res {
            res_json.bio = Some(bio.to_owned());
        }
    }
    if let Some(username) = &lmaobozo.0[2] {
        let res = sqlx::query!("UPDATE users SET username=$1 WHERE id=$2", username, identity.id)
            .execute(DB_POOL.get().await).await;
        if let Ok(_res) = res {
            res_json.username = Some(username.to_owned());
        }
    }
    
    Ok(HttpResponse::Ok()
        .json(res_json))
}

// user_change_password, user_revoke_tokens

#[derive(Serialize)]
struct WG {
    id : i32,
    url: String,

    name: String,
    description: String,

    profile_pic: Option<DBRetrUpload>,
    header_pic: Option<DBRetrUpload>
}

#[derive(Serialize)]
struct User {
    id : i32,
    username: String,

    name: String,
    bio: String,

    profile_pic: Option<DBRetrUpload>,
}

#[get("/my_wg")]
async fn get_wg(identity: Identity) -> Result<impl Responder, Error> {
    let wgopt =
    if let Some(wg_id)  = identity.wg {
        //let wg = sqlx::query_as!(WG, "SELECT * FROM wgs WHERE id = $1", wg_id)
        let wg : WG = sqlx::query_as!(WG, r#"SELECT wgs.id, url, name, description, 
        (pp.id, pp.extension, pp.original_filename, pp.size_kb) as "profile_pic: DBRetrUpload",
        (hp.id, hp.extension, hp.original_filename, hp.size_kb) as "header_pic: DBRetrUpload"
    FROM wgs 
    LEFT JOIN uploads AS pp ON profile_pic = pp.id
    LEFT JOIN uploads AS hp ON header_pic = hp.id
    WHERE wgs.id = $1"#, wg_id)
            .fetch_one(DB_POOL.get().await).await.map_err(|e| {error!("AHH: {}", e); res_error(StatusCode::INTERNAL_SERVER_ERROR, Some(e), "Database quirked up, sry :(")})?;
        Some(wg)
    } else {
        None
    };
    
    Ok( HttpResponse::Ok()
    .json(wgopt) )
}

#[put("/my_wg")]
async fn put_wg(identity: Identity, payload: Multipart) -> Result<impl Responder, Error> {
    let wg_id = identity.wg.ok_or_else(|| res_error::<&'static str>(StatusCode::FORBIDDEN, None, "You are not assigned to a WG, and therefore can't edit yours.") )?;

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
        let new_upl = change_upload!("wgs", "profile_pic", i32)(profile_picf.move_responsibility(), wg_id).await;
        if let Ok (new_upl) = new_upl  {
            res_json.profile_pic = Some(new_upl);
        } else {
            warn!("Couldn't change upload :(");
        }
    }
    if let Some(header_picf) = &mut lmaobozo.1[1] {
        let new_upl = change_upload!("wgs", "header_pic", i32)(header_picf.move_responsibility(), wg_id).await;
        if let Ok (new_upl) = new_upl  {
            res_json.header_pic = Some(new_upl);
        } else {
            warn!("Couldn't change upload :(");
        }
    }
    if let Some(name) = &lmaobozo.0[0] {
        let res = sqlx::query!("UPDATE wgs SET name=$1 WHERE id=$2", name, wg_id)
            .execute(DB_POOL.get().await).await;
        if let Ok(_res) = res {
            res_json.name = Some(name.to_owned());
        }
    }
    if let Some(url) = &lmaobozo.0[1] {
        let res = sqlx::query!("UPDATE wgs SET url=$1 WHERE id=$2", url, wg_id)
            .execute(DB_POOL.get().await).await;
        if let Ok(_res) = res {
            res_json.url = Some(url.to_owned());
        }
    }
    if let Some(description) = &lmaobozo.0[2] {
        let res = sqlx::query!("UPDATE wgs SET description=$1 WHERE id=$2", description, wg_id)
            .execute(DB_POOL.get().await).await;
        if let Ok(_res) = res {
            res_json.description = Some(description.to_owned());
        }
    }
    
    Ok(HttpResponse::Ok()
        .json(res_json))
}

#[get("/my_wg/users")]
async fn get_wg_users(identity: Identity) -> Result<impl Responder, Error>  {
    let wgopt =
    if let Some(wg_id)  = identity.wg {
        //let wg = sqlx::query_as!(WG, "SELECT * FROM wgs WHERE id = $1", wg_id)
        let wg : Vec<User> = sqlx::query_as!(User, r#"SELECT users.id, name, bio, username, 
        (pp.id, pp.extension, pp.original_filename, pp.size_kb) as "profile_pic: DBRetrUpload"
    FROM users 
    LEFT JOIN uploads AS pp ON profile_pic = pp.id
    WHERE users.wg = $1"#, wg_id)
            .fetch_all(DB_POOL.get().await).await.map_err(|e| {error!("AHH: {}", e); res_error(StatusCode::INTERNAL_SERVER_ERROR, Some(e), "Database quirked up, sry :(")})?;
        Some(wg)
    } else {
        None
    };
    
    Ok( HttpResponse::Ok()
    .json(wgopt) )
}

#[get("/my_wg/costs")]
async fn get_wg_costs(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[post("/my_wg/costs")]
async fn post_wg_costs(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[get("/my_wg/costs/{id}")]
async fn get_wg_costs_id(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[put("/my_wg/costs/{id}")]
async fn put_wg_costs_id(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[delete("/my_wg/costs/{id}")]
async fn delete_wg_costs_id(identity: Identity) -> impl Responder {
    todo!();
    ""
}

#[get("/my_wg/costs/stats")]
async fn get_wg_costs_stats(identity: Identity) -> impl Responder {
    todo!();
    ""
}


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            //.wrap(Authentication)
            .service(get_user_me)
            .service(put_user_me)
            .service(get_wg)
            .service(put_wg)
            .service(get_wg_users)
            .service(get_wg_costs)
            .service(post_wg_costs)
            .service(get_wg_costs_id)
            .service(put_wg_costs_id)
            .service(delete_wg_costs_id)
            .service(get_wg_costs_stats)

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