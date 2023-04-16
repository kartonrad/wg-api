use dioxus::prelude::*;
use futures_lite::stream::StreamExt;
use serde::{Serialize,Deserialize};

use crate::{LoggedInApp, LoggedOutApp};
use crate::constants::*;

#[derive(Serialize)]
pub struct Credentials {
    username: String,
    password: String
}

pub enum IdentityEvent {
    FetchWg,
    FetchWgHeader,
    FetchWgProfilePic,
    FetchProfilePic
}

#[derive(PartialEq)]
pub struct Identity;

#[derive(Clone, PartialEq)]
pub struct Token;

pub enum LoginEvent {
    Login(Credentials), // login user and switch to it
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
            //.https_only(true)
            .build().expect("To be able to make requests.");

        async move {

            while let Some(msg) = rx.next().await {
                match msg {
                    LoginEvent::Login(cred) => { 
                        // get new identity
                        let new = Token;

                        let response = try_c!( 
                            client.post(  format!("{}/login", API_URL)  )
                                .body( try_c!(serde_json::to_string( &cred )) )
                                .send().await
                        );

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
        render!( SomeWrapper { identity: ident.clone() } )
    } else {
        render!( LoggedOutApp {} )
    }
}

#[inline_props]
pub fn SomeWrapper(cx: Scope, identity: Token) -> Element {

    render!(
        LoggedInApp {
            identity: Identity
        }
    )
}


pub fn use_identity_provider(cx: Scope) {
    use_shared_state_provider::<Option<Identity>>(&cx, || None );
    
    async fn profile_service ( rx: UnboundedReceiver<IdentityEvent> ) {

    }
}

