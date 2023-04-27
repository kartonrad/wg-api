
use std::collections::HashMap;

use common::{auth::IIdentity, Cost, CostShare, DBUpload, RegularDef, RegularSpending, Upload, User, UserDebt, WG};
use serde::{Deserialize, Serialize};
use common::auth::SerdeIdentity;
use crate::constants::API_URL;

pub type HTTP = reqwest::Client;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WGMember {
    pub identity : IIdentity,
    pub wg: WG,
    pub friends: HashMap<i32, User>
}

pub fn upload_to_path(opt_upload: Option<DBUpload> ) -> Option<String> {
    if let Some(header) = opt_upload {
        let header : Option<Upload> = header.into();
        if let Some(header) = header {
            return Some(header.into_url());
        }
    }
    
    None
}

// REQUESTS
pub async fn get_member(client: reqwest::Client) -> Result<WGMember, reqwest::Error> {
    let identity: SerdeIdentity = client.get( format!("{}/api/me", API_URL) ).send().await?
        .json().await?;

    let wg: WG = client.get( format!("{}/api/my_wg", API_URL) ).send().await?
        .json().await?;

    let mut friendsVec: Vec<User> = client.get( format!("{}/api/my_wg/users", API_URL) ).send().await?
        .json().await?;
    let mut friends = HashMap::new();
    friendsVec.drain(..).for_each( | fr | {
        friends.insert(fr.id, fr);
    });

    return Ok(WGMember {
        identity: identity.into(),
        wg,
        friends
    });
}

#[macro_export]
/// This is a macro to simplify calling async functions defined in wg_app::api.
///
/// it takes in the function name as first parameter, followed by a semicolon,
/// 2 other expressions seperated by commas.
/// these are: a dioxus component Scope, and a handle to a reqwest::Client.
/// following that is an optional list of dependencies.
/// these are names of variables, that are passed into the api,
/// and a new request is made every time these change.
///
/// it runs the future in the dioxus component scope specified by the second parameter,
/// passing all the remaining parameters to the api function.
/// *DO NOT RUN THIS MACRO CONDITIONALLY! IT INVOKES A HOOK!*
/// ```rust
/// fn Component(cx: Scope) -> Element {
///     let http = use_context::<HTTP>(cx)?;
///
///     let costs = use_api_else_return!(get_costs; cx, http.clone());
///
///     // we now have own a Vec<Cost>
///     // in case of error, or the loading state, nothing is rendered.
///     // that might be undesirable for your use case
///     // TODO: make the macro render a generic error message!
/// }
/// ```
macro_rules! use_api_else_return {
    ($func_name : ident; $cx:expr, $http:expr $(, $param:ident)*) => {
        {
            use dioxus::prelude::use_future;
            use crate::api::$func_name;

             let val =
                use_future( $cx, &( $($param,)*), move |( $($param,)*)| {
                    $func_name($http.clone(), $($param,)*)
                });
            let val = val.value()?.to_owned()?;

            val
        }
    };
}

// Functions intended to be used with the macro:


pub async fn get_costs(http: HTTP) -> Option<Vec<Cost>> {
    Some(
        http.get( format!("{API_URL}/api/my_wg/costs") ).send().await.ok()?
            .json().await.ok()?
    )
}

pub async fn get_cost(http: HTTP, id: i32) -> Option<Cost> {
    Some(
        http.get( format!("{API_URL}/api/my_wg/costs/{id}/detail") ).send().await.ok()?
            .json::<Option<Cost>>().await.ok()??
    )
}

pub async fn get_shares(http: HTTP, id: i32) -> Option<Vec<CostShare>> {
    Some(
        http.get( format!("{API_URL}/api/my_wg/costs/{id}/shares") ).send().await.ok()?
            .json::<Vec<CostShare>>().await.ok()?
    )
}

// Attention: in the app i call /stats "the Tally", and /over_time "the stats"
// because that makes way more sense now that i thought of the word "tally"
pub async fn get_tally(http: HTTP, id: Option<i32>) -> Option<Vec<UserDebt>> {
    let qry = if let Some(id) = id {format!("?balance={id}")} else {String::from("")};

    Some (
        http.get( format!("{API_URL}/api/my_wg/costs/stats{qry}") ).send().await.ok()?
            .json::<Vec<UserDebt>>().await.ok()?
    )
}

async fn get_stats(http: HTTP, period: RegularDef) -> Option<Vec<RegularSpending>> {
    Some (
        http.get( format!("{API_URL}/api/my_wg/costs/over_time/{period}") ).send().await.ok()?
            .json::<Vec<RegularSpending>>().await.ok()?
    )
}
