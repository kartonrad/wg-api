#![allow(non_snake_case)]
use common::{auth::{IIdentity, LoginInfo}, WG};
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::{
    prelude::*
};
use dioxus_router::{Route, Router, Redirect, Link};

mod identity_service;
mod constants;
pub mod network_types;
use identity_service::LoginEvent;
use network_types::WGMember;

fn main() {
    // launch the web app
    dioxus_web::launch(App);
}

// create a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    render!(
        style { include_str!("../src/style.css") }
    
        identity_service::IdentityProvider {}
    )
}

// Identity Provider calls this 
pub fn LoggedOutApp(cx: Scope) -> Element {
    render!(
        "logged out"
        SketchyLoginForm {}
    )
}

pub fn SketchyLoginForm(cx: Scope) -> Element {
    let login_handle = use_coroutine_handle::<LoginEvent>(cx).expect("SketchyLoginForm only runs under IdentityProvider (getting coroutine handle fialed)");

    let when_submit = |ev : FormEvent| {
        ev.stop_propagation(); 
        let info = || -> Option<LoginInfo> {
            Some(LoginInfo {
                username: ev.values.get("username")?.to_owned(),
                password: ev.values.get("password")?.to_owned()
            })
        }();
        
        if let Some(info) = info {
            login_handle.send(LoginEvent::Login(info));
        }
    };

    render!(
        form {
            class: "login_form",
            prevent_default: "onsubmit",
            onsubmit: when_submit,

            label {
                class: "username",

                "Username: "
                input {
                    r#type: "text",
                    name: "username",
                    placeholder: "mustermann"
                }
            }
            br {}

            label {
                class: "password",

                "Password: "
                input {
                    r#type: "password",
                    name: "password",
                    placeholder: "***"
                }
            }
            br {}

            input { 
                r#type: "submit",
                value: "Login" 
            }
        }
    )
}

// Identity Provider also  calls this 
#[inline_props]
pub fn LoggedInApp<'a>(cx: Scope, member: &'a WGMember) -> Element {
    to_owned![member];
    use_shared_state_provider(cx, || member.clone()); // finally, globally share member - it can now be edited from anywere below in the tree

    cx.render(rsx! {
        Router {
            Route { to: "/home",     HomeScreen   {} BottomTabs {} }
            Route { to: "/chores",   ChoreScreen  {} BottomTabs {} }
            Route { to: "/costs",    CostScreen   {} BottomTabs {} }
            Route { to: "/settings", SettingScreen{} BottomTabs {} }
            Redirect { from: "", to: "/home" }
        }
    })
}

fn HomeScreen(cx: Scope) -> Element {
    cx.render(rsx!( div { "Hello World 🦀" }))
}
fn ChoreScreen(cx: Scope) -> Element {
    cx.render(rsx!( div { "CHORE" }))
}
fn CostScreen(cx: Scope) -> Element {
    cx.render(rsx!( div { "COSTS" }))
}
fn SettingScreen(cx: Scope) -> Element {
    cx.render(rsx!( div { "Settings" }))
}


fn BottomTabs(cx: Scope) -> Element {
    
    cx.render(rsx!(
        nav {
            class: "bottom_tabs",

            Link { to: "/home",    span {"💒"} }
            Link { to: "/chores",  span {"🧹"} }
            Link { to: "/costs",   span {"💵"} }
            Link { to: "/settings",span {"⚙️"} }
        }
    ))
}