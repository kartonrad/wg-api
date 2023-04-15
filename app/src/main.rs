#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::{
    prelude::*
};
use dioxus_router::{Route, Router, Redirect, Link};

mod identity_service;
mod constants;

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

// Identity Provider calls these
pub fn LoggedOutApp(cx: Scope) -> Element {
    render!(
        "logged out"
    )
}

#[inline_props]
pub fn LoggedInApp(cx: Scope, identity: identity_service::Identity) -> Element {
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