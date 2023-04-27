use dioxus::prelude::*;
use crate::API_URL;
use crate::identity_service::{upload_to_path, WGMember};

pub fn HomeScreen(cx: Scope) -> Element {
    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();
    let header = upload_to_path( member.wg.header_pic.clone()).unwrap_or("/public/img/rejection.jpg".to_string());
    let profile_pic = upload_to_path( member.wg.profile_pic.clone()).unwrap_or("/public/img/rejection.jpg".to_string());

    let userelems = member.friends.iter().map(|(_uid, user)| {
        let profile_pic = upload_to_path( user.profile_pic.clone() ).unwrap_or("/public/img/rejection.jpg".to_string());
        rsx!(
            div {
                class:"user_card",
                key: "{user.id}",

                div {
                    class: "avatar",
                    background_image: "url({API_URL}{profile_pic})",
                }

                h2 { "{user.name}" }
                h4 { "@{user.username}" }
                span { "{user.bio}" }
            }
        )
    });

    render!(
        div {
            background_image: "url({API_URL}{header})",
            class: "wg_header",

            div {
                class: "wg_avatar",
                background_image: "url({API_URL}{profile_pic})",
            }
        }
        div {
            class: "wg_body",

            h3 { "{member.wg.name}"}  
            p {
                "{member.wg.description}"
            }
        }
        div {
            class: "scroll_container", 
            userelems
        }
    )
}