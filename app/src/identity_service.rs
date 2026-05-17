use dioxus::prelude::*;
use dioxus_sdk::storage::use_persistent;
use futures_lite::stream::StreamExt;
use reqwest::header::HeaderMap;

use crate::constants::*;
use crate::{LoggedInApp, LoggedOutApp};

use common::auth::*;
use common::{DBUpload, Upload, User, WG};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub enum LoginEvent {
    Login(LoginInfo),       // login user and switch to it
    Logout(String), // remove login from list, and switch to none if that was the selected one
    Switch(Option<String>), // switch to none or user specified by string
}

macro_rules! try_c {
    ($e: expr) => {
        if let Ok(v) = $e {
            v
        } else {
            continue;
        }
    };
}

pub fn IdentityProvider() -> Element {
    let identity: Signal<Option<Token>> = use_persistent("identity_token", || None::<Token>);
    let other_identities = use_signal(|| Vec::<Token>::new());
    let identity2 = identity.clone();

    let _service = use_coroutine(move |mut rx: UnboundedReceiver<LoginEvent>| {
        to_owned![other_identities, identity];

        let client = reqwest::Client::builder()
            // NOT AVAILABLE IN WASM??
            //.user_agent( concat!(env!("CARGO_PKG_NAME"),"/",env!("CARGO_PKG_VERSION"), ) )
            //.https_only(true)
            .build()
            .expect("To be able to make requests.");

        async move {
            while let Some(msg) = rx.next().await {
                match msg {
                    LoginEvent::Login(cred) => {
                        // get new identity
                        let response = try_c!(
                            client
                                .post(format!("{}/auth/login", API_URL))
                                .json(&cred)
                                .send()
                                .await
                        );
                        let new: Token = try_c!(response.json().await);

                        // store away old identity
                        if let Some(before) = identity.with_mut(|i| i.take()) {
                            other_identities.with_mut(|i| i.push(before))
                        }

                        // set new
                        identity.set(Some(new));
                    }
                    LoginEvent::Logout(_) => todo!(),
                    LoginEvent::Switch(_) => todo!(),
                }
            }
        }
    });

    if let Some(ident) = identity2.cloned() {
        rsx!(SomeWrapper {
            token: ident.clone()
        })
    } else {
        rsx!(LoggedOutApp {})
    }
}

#[component]
pub fn SomeWrapper(token: Token) -> Element {
    // Responsible for providing the global client for authenticated requests!
    let mut headers = HeaderMap::new();
    headers.append(
        reqwest::header::AUTHORIZATION,
        format!("Bearer {}", token.token).parse().unwrap(),
    ); // EVIL UNWRAP!!

    let meclient = use_context_provider(
        move || {
            reqwest::Client::builder()
                .default_headers(headers)
                // NOT AVAILABLE IN WASM??
                //.https_only(true)
                //.user_agent( concat!(env!("CARGO_PKG_NAME"),"/",env!("CARGO_PKG_VERSION"), ) )
                .build()
                .unwrap()
        }, // succeeds if this supports tls
    );

    // Responsible for loading the WG - checking
    let member = use_resource(move || {
        let meclient = meclient.clone(); // inefficient???? no it uses an internal reference counter!! (banger)
        get_member(meclient)
    });

    if let Some(ref member) = *member.read() {
        match member {
            Ok(member) => {
                return rsx!(LoggedInApp {
                    member: member.clone()
                });
            }
            Err(e) => {
                // handle the Error that user is in no wg - with explanation
                return rsx!( "Error occured.\nSomething might not have gone through,\nor you aren't member of any WG...\nWhich we should probably detect and handle lol\nBut no\n {e:?}" );
            }
        }
    }

    rsx!("Opening WG...")
}

// REQUESTS
pub async fn get_member(client: reqwest::Client) -> Result<WGMember, reqwest::Error> {
    let identity: SerdeIdentity = client
        .get(format!("{}/api/me", API_URL))
        .send()
        .await?
        .json()
        .await?;

    let wg: WG = client
        .get(format!("{}/api/my_wg", API_URL))
        .send()
        .await?
        .json()
        .await?;

    let mut friendsVec: Vec<User> = client
        .get(format!("{}/api/my_wg/users", API_URL))
        .send()
        .await?
        .json()
        .await?;
    let mut friends = HashMap::new();
    friendsVec.drain(..).for_each(|fr| {
        friends.insert(fr.id, fr);
    });

    return Ok(WGMember {
        identity: identity.into(),
        wg,
        friends,
    });
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WGMember {
    pub identity: IIdentity,
    pub wg: WG,
    pub friends: HashMap<i32, User>,
}

pub fn upload_to_path(opt_upload: Option<DBUpload>) -> Option<String> {
    if let Some(header) = opt_upload {
        let header: Option<Upload> = header.into();
        if let Some(header) = header {
            return Some(header.into_url());
        }
    }

    None
}
