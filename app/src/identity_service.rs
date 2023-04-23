use std::collections::HashMap;

use dioxus::prelude::*;
use futures_lite::stream::StreamExt;
use reqwest::header::HeaderMap;

use crate::network_types::WGMember;
use crate::{LoggedInApp, LoggedOutApp};
use crate::constants::*;

use common::{auth::*, WG, User};

pub enum IdentityEvent {
    FetchWg,
    FetchWgHeader,
    FetchWgProfilePic,
    FetchProfilePic
}
pub enum LoginEvent {
    Login(LoginInfo), // login user and switch to it
    Logout(String), // remove login from list, and switch to none if that was the selected one
    Switch(Option<String>) // switch to none or user specified by string
}

macro_rules! try_c {
    ($e: expr) => {
        if let Ok(v) = $e {v} else {  continue;}
    };
}

pub fn IdentityProvider(cx: Scope) -> Element {
    
    let other_identities = use_ref(&cx, || Vec::<Token>::new());
    let identity = use_ref(&cx, || None::<Token>);


    let _service = use_coroutine(&cx, |mut rx: UnboundedReceiver<LoginEvent>| {
        to_owned![other_identities, identity];
        
        let client = reqwest::Client::builder()
            // NOT AVAILABLE IN WASM??
            //.user_agent( concat!(env!("CARGO_PKG_NAME"),"/",env!("CARGO_PKG_VERSION"), ) )
            //.https_only(true)
            .build().expect("To be able to make requests.");

        async move {

            while let Some(msg) = rx.next().await {
                match msg {
                    LoginEvent::Login(cred) => { 
                        // get new identity
                        let response = try_c!( 
                            client.post(  format!("{}/auth/login", API_URL)  )
                                .json( &cred )
                                .send().await
                        );
                        let new : Token = try_c!(response.json().await);

                        // store away old identity
                        if let Some(before) = identity.with_mut(|i| i.take()) {
                            other_identities.with_mut( |i| i.push(before) ) 
                        }
                        
                        // set new
                        identity.set(Some(new));
                    },
                    LoginEvent::Logout(_) => todo!(),
                    LoginEvent::Switch(_) => todo!(),
                }
            }
        }
    });
    
    if let Some(ident) = &*identity.read() {
        render!( SomeWrapper { token: ident.clone() } )
    } else {
        render!( LoggedOutApp {} )
    }
}

async fn get_member(client: reqwest::Client) -> Result<WGMember, reqwest::Error> {
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

#[inline_props]
pub fn SomeWrapper(cx: Scope, token: Token) -> Element {
    // Responsible for providing the global client for authenticated requests!
    let mut headers = HeaderMap::new();
    headers.append(reqwest::header::AUTHORIZATION, format!( "Bearer {}", token.token ).parse().unwrap() ); // EVIL UNWRAP!!

    let meclient = use_context_provider(cx, 
        move || reqwest::Client::builder()
            .default_headers(headers)
            // NOT AVAILABLE IN WASM??
            //.https_only(true)
            //.user_agent( concat!(env!("CARGO_PKG_NAME"),"/",env!("CARGO_PKG_VERSION"), ) )
            .build().unwrap() // succeeds if this supports tls
    );

    // Responsible for loading the WG - checking
    let member = use_future(cx, (token,), |_| {
        let meclient = meclient.clone(); // inefficient???? no it uses an internal reference counter!! (banger)
        get_member(meclient)
    });

    if let Some(member) = member.value() {
        match member { 
            Ok(member) => {
                return render!(
                    LoggedInApp {
                        member: member
                    }
                );
            },
            Err(e) => { 
                // handle the Error that user is in no wg - with explanation
                return render!( "Error occured.\nSomething might not have gone through,\nor you aren't member of any WG...\nWhich we should probably detect and handle lol\nBut no\n {e:?}" )
            }
        }
    }
    
    render!(
        "Opening WG..."
    )
}

/* 
pub fn use_identity_provider(cx: Scope) {
    use_shared_state_provider::<Option<Identity>>(&cx, || None );
    
    async fn profile_service ( rx: UnboundedReceiver<IdentityEvent> ) {

    }
}

*/