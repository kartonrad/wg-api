#![allow(non_snake_case)]
use common::auth::LoginInfo;
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::{
    prelude::*
};
use dioxus_router::{Route, Router, Redirect, Link, use_router};
use log::Level;

mod identity_service;
mod constants;
pub mod network_types;
pub mod screens;
use screens::{home::HomeScreen, costs::{CostEntryScreen, CostDetailScreen}, chores::ChoreScreen, settings::SettingScreen};

use constants::API_URL;
use identity_service::LoginEvent;
use network_types::{WGMember, get_upload};

fn main() {
    // launch the web app
    #[cfg(feature = "web")]{
        console_log::init_with_level(Level::Trace).expect("Logging to initialize??");
        dioxus_web::launch(App);
    }
    #[cfg(feature = "desktop")]
    {
        pretty_env_logger::init();
        dioxus_desktop::launch(App);
    }
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
        let info = (|| -> Option<LoginInfo> {
            Some(LoginInfo {
                username: ev.values.get("username")?.to_owned(),
                password: ev.values.get("password")?.to_owned()
            })
        })();
        
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

    render!(
        Router {
            Route { to: "/home",     Layout { HomeScreen  {} }  } // BottomTabs need to be in here for links to work
            Route { to: "/chores",   Layout { ChoreScreen  {} }  }
            Route { to: "/costs",    Layout { CostEntryScreen  {} }  }
            Route { to: "/costs/detail", Layout { CostDetailScreen  {} }  }
            Route { to: "/settings", Layout { SettingScreen  {} }  }
            Redirect { from: "", to: "/home" }
        }
    )
}

#[inline_props]
pub fn HeaderBar<'a>(cx: Scope, title: &'a str) -> Element {
    let router = use_router(cx);

    render!(
        nav {
            class: "header_bar",

            a {
                onclick: |_| {
                    router.pop_route();
                },

                "⬅️"
            }
            h2 { "{title}" }
        }
    )
}

#[inline_props]
pub fn Layout<'a>(cx: Scope, children: Element<'a>) -> Element {
    let member = use_shared_state::<WGMember>(cx).unwrap();

    let upl = get_upload( member.read().wg.header_pic.clone()).unwrap_or("/public/img/rejection.jpg".to_string());

    render!(
        div {
            class: "wg_app_background",
            background_image: "url({API_URL}{upl})",

            children
        }
        BottomTabs {}
    )
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