#![allow(non_snake_case)]
use anyhow::{anyhow, bail};
use common::auth::LoginInfo;
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
use dioxus_router::{router, use_router, Link, Outlet, Routable, Router};
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
        launch(App);
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
        style {
            {include_str!("../dist/post-style.css")}
        }

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
    let login_handle = use_coroutine_handle::<LoginEvent>(); /* .expect(
                                                                 "SketchyLoginForm only runs under IdentityProvider (getting coroutine handle fialed)",
                                                             );*/
    let when_submit = move |ev: FormEvent| async move {
        let values = ev.values();

        let username = values.iter().find(|entry| entry.0 == "username");
        let username = match username {
            Some((_, FormValue::Text(username))) => username,
            _ => {
                info!("Username field is missing!");
                return;
            }
        };

        let password = values.iter().find(|entry| entry.0 == "password");
        let password = match password {
            Some((_, FormValue::Text(password))) => password,
            _ => {
                info!("Password field is missing!");
                return;
            }
        };

        let info = (|| -> Option<LoginInfo> {
            Some(LoginInfo {
                username: username.to_string(),
                password: password.to_string(),
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

#[rustfmt::skip]
#[derive(Clone, Debug, PartialEq, Routable)]
enum Route {
    #[layout(Layout)]
    #[route("/chores")]
    ChoreScreen,

    #[nest("/costs")]

        #[route("/new")]
        CostNewScreen {},

        #[route("/detail")]
        CostDetailScreen {},


        #[route("/balance")]
        CostBalanceDetailScreen {},

        #[layout(TopTabs)]
        #[route("/")]
        CostListScreen {},
        #[route("/tally")]
        CostTallyScreen {},
        #[route("/stats")]
        CostStatScreen {},

    #[end_nest]

    #[route("/")]
    HomeScreen,
}

// Identity Provider also  calls this
#[component]
pub fn LoggedInApp(member: WGMember) -> Element {
    to_owned![member];
    use_context_provider(|| Signal::new(member.clone())); // finally, globally share member - it can now be edited from anywere below in the tree

    rsx!(Router::<Route> {})
}

#[component]
fn TopTabs() -> Element {
    rsx!(
        nav {
            class: "top_tabs",

            Link { to: "/costs",  span {"Einträge"} }
            Link { to: "/costs/tally",   span {"Stand"} }
            Link { to: "/costs/stats",span {"Statistik"} }
        }
        Outlet::<Route> {}
    )
}

#[component]
pub fn HeaderBar(title: String) -> Element {
    let router = router();

    rsx!(
        nav {
            class: "header_bar",

            a {
                onclick: move |_| {
                    //router.pop_route();
                    router.go_back();
                },

                "⬅️"
            }
            h2 { "{title}" }
        }
    )
}

#[component]
pub fn Layout() -> Element {
    let member = use_context::<Signal<WGMember>>();

    let upl = upload_to_path(member.read().wg.header_pic.clone())
        .unwrap_or("/public/img/rejection.jpg".to_string());

    rsx!(
        div {
            class: "wg_app_background",
            background_image: "url({API_URL}{upl})",

            Outlet::<Route> {}
        }
        BottomTabs {}
    )
}

fn BottomTabs() -> Element {
    rsx!(
        nav {
            class: "bottom_tabs",

            Link { to: "/",    span {"💒"} }
            Link { to: "/chores",  span {"🧹"} }
            Link { to: "/costs",   span {"💵"} }
            Link { to: "/settings",span {"⚙️"} }
        }
    )
}
