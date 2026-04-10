#![allow(non_snake_case)]
use common::auth::LoginInfo;
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
use dioxus_router::{use_router, Link, Redirect, Route, Router};
use log::Level;

pub mod api;
mod constants;
mod identity_service;
pub mod screens;
mod time;

use screens::{chores::ChoreScreen, costs::*, home::HomeScreen, settings::SettingScreen};

use constants::API_URL;
use identity_service::{upload_to_path, LoginEvent, WGMember};

fn main() {
    // launch the web app
    #[cfg(feature = "web")]
    {
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
fn App() -> Element {
    rsx!(
        style { include_str!("../dist/post-style.css") }

        identity_service::IdentityProvider {}
    )
}

// Identity Provider calls this
pub fn LoggedOutApp() -> Element {
    rsx!(
        "logged out"
        SketchyLoginForm {}
    )
}

pub fn SketchyLoginForm() -> Element {
    let login_handle = use_coroutine_handle::<LoginEvent>().expect(
        "SketchyLoginForm only runs under IdentityProvider (getting coroutine handle fialed)",
    );

    let when_submit = |ev: FormEvent| {
        ev.stop_propagation();
        let info = (|| -> Option<LoginInfo> {
            Some(LoginInfo {
                username: ev.values.get("username")?.to_owned(),
                password: ev.values.get("password")?.to_owned(),
            })
        })();

        if let Some(info) = info {
            login_handle.send(LoginEvent::Login(info));
        }
    };

    rsx!(
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
pub fn LoggedInApp(member: WGMember) -> Element {
    to_owned![member];
    use_shared_state_provider(cx, || member.clone()); // finally, globally share member - it can now be edited from anywere below in the tree

    rsx!(
        Router {
            Route { to: "/home",     Layout { HomeScreen  {} }  } // BottomTabs need to be in here for links to work
            Route { to: "/chores",   Layout { ChoreScreen  {} }  }

            Route { to: "/costs",    Layout { TopTabs {} CostListScreen {} }  }
            Route { to: "/costs/new",    Layout { CostNewScreen {} }  }
            Route { to: "/costs/detail", Layout { CostDetailScreen  {} }  }
            Route { to: "/costs/tally", Layout { TopTabs {} CostTallyScreen {} }  }
            Route { to: "/costs/balance", Layout { CostBalanceDetailScreen {} }  }
            Route { to: "/costs/stats", Layout { TopTabs {}  CostStatScreen  {} }  }

            Route { to: "/settings", Layout { SettingScreen  {} }  }
            Redirect { from: "", to: "/home" }
        }
    )
}

fn TopTabs() -> Element {
    cx.render(rsx!(
        nav {
            class: "top_tabs",

            Link { to: "/costs",  span {"Einträge"} }
            Link { to: "/costs/tally",   span {"Stand"} }
            Link { to: "/costs/stats",span {"Statistik"} }
        }
    ))
}

#[component]
pub fn HeaderBar(title: String) -> Element {
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

pub fn Layout(children: Element) -> Element {
    let member = use_shared_state::<WGMember>(cx).unwrap();

    let upl = upload_to_path(member.read().wg.header_pic.clone())
        .unwrap_or("/public/img/rejection.jpg".to_string());

    render!(
        div {
            class: "wg_app_background",
            background_image: "url({API_URL}{upl})",

            children
        }
        BottomTabs {}
    )
}

fn BottomTabs() -> Element {
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

