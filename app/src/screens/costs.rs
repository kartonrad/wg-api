use common::Cost;
use dioxus::prelude::*;
use dioxus_router::{Link, use_route};
use rust_decimal::Decimal;
use crate::{network_types::{HTTP, WGMember, get_upload}, constants::API_URL};
use time::macros::format_description;

async fn get_costs(http: HTTP) -> Option<Vec<Cost>> {
    Some(
        http.get( format!("{}/api/my_wg/costs", API_URL) ).send().await.ok()?
            .json().await.ok()?
    )
}

pub fn CostScreen(cx: Scope) -> Element {


    render!(
        Router {
            Route { to: "/",       EntryScreen {}  }
            Route { to: "/detail", DetailScreen {}  }
        }
    )
}

fn EntryScreen(cx: Scope) -> Element {
    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();
    let http = use_context::<HTTP>(cx)?;

    let costs =
    use_future( cx, (), move |_| {  
        get_costs(http.clone())
    });
    let costs = costs.value()?.to_owned()?;

    render!( 
        div {
            class: "scroll_container",

            costs.iter().map(|c|
                rsx!(
                    CostEntry {
                        c: c.clone(),
                    }
                )
            )
        }
    )
}

fn DetailScreen(cx: Scope)-> Element {
    let route = use_route(cx);
    let id = route.query::<i32>()?;


}



// Cost Object
#[inline_props]
fn CostEntry(cx: Scope, c: Cost) -> Element {
    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();
    let interpreted = interpret_cost(member.identity.id, &c)?;

    let user = &member.friends[&c.creditor_id];
    let profile_pic = get_upload(user.profile_pic.clone()).unwrap_or("".to_string());


    let amt = c.amount.round_dp(2);
    let date = c.added_on.format(&format_description!("[day] [month repr:short]")).expect("EE");    

    render!(
        Link {
            to : "/costs/detail?id={c.id}",
            class: "nolink",

            div {
                class: "cost_card",
                key: "{c.id}",
    
                div {
                    class: "body",
    
                    h4 { "{c.name}" }
                
                    span { 
                        div {
                            class: "avatar",
                            background_image: "url({API_URL}{profile_pic})",
                        }
                        i {"{user.name}"} " bezahlte {amt} €"
                    }
                }
                div {
                    class: "left",

                    AmountDisplay {
                        amt: interpreted.my_gain
                    }
                    br {}
                    span { "{date}" }
                }
            }
        }
    )
}

struct InterpretedCost {
    /// how much you pay or are paid when this cost is balanced
    my_gain: Decimal,
    /// how much one share is worth
    single_payment: Decimal,
    // the costs amount field, again
    amt: Decimal,
    /// whether this user is the creditor 
    am_creditor: bool
}

fn interpret_cost(me_id: i32, cost: &Cost) -> Option<InterpretedCost> {
    let amt = cost.amount;
    let nr_unpaid_shares = Decimal::from(cost.nr_unpaid_shares?);
    let nr_shares = Decimal::from(cost.nr_shares?);
    let my_share = cost.my_share.clone()?;

    let repayment_fract = nr_unpaid_shares / nr_shares;
    let repayment = repayment_fract * amt;
    let single_payment = amt / nr_shares;

    let mut my_gain = Decimal::ZERO;

    let my_share_paid = my_share.paid == Some(true) || my_share.paid == None;

    let am_creditor = me_id == cost.creditor_id;
    if am_creditor {
        my_gain += repayment;
    } else {
        my_gain -= if my_share_paid { Decimal::ZERO } else { single_payment };
    }

    return Some(InterpretedCost {my_gain, single_payment, amt, am_creditor});
}

#[inline_props]
pub fn AmountDisplay ( cx: Scope, amt: Decimal ) -> Element {
    let mut amtstr = format!("{amt:.2}");
    if amt.is_sign_positive() {
        amtstr.insert(0, '+');
    }

    let class =
        if amt.is_zero() {
            "amount_display zero"
        } else {
            if amt.is_sign_positive() { "amount_display positive" }
            else { "amount_display negative" }
        };

    render!(
        span {
            class: class,

            "{amtstr}"
            span {
                class: "ccy",
                "€"
            }
        }
    )
}