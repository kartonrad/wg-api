use common::{Cost, CostShare};
use dioxus::prelude::*;
use dioxus_router::{Link, Redirect, Route, Router, use_route, use_router};
use log::trace;
use rust_decimal::Decimal;
use serde::Deserialize;
use crate::{network_types::{HTTP, WGMember, get_upload}, constants::API_URL, HeaderBar};
use time::macros::format_description;

async fn get_costs(http: HTTP) -> Option<Vec<Cost>> {
    Some(
        http.get( format!("{API_URL}/api/my_wg/costs") ).send().await.ok()?
            .json().await.ok()?
    )
}

async fn get_cost(http: HTTP, id: i32) -> Option<Cost> {
    Some(
        http.get( format!("{API_URL}/api/my_wg/costs/{id}/detail") ).send().await.ok()?
            .json::<Option<Cost>>().await.ok()??
    )
}

async fn get_shares(http: HTTP, id: i32) -> Option<Vec<CostShare>> {
    Some(
        http.get( format!("{API_URL}/api/my_wg/costs/{id}/shares") ).send().await.ok()?
            .json::<Vec<CostShare>>().await.ok()?
    )
}

/*
pub fn CostScreen(cx: Scope) -> Element {
    render!(
        Router {
            Redirect { from: "", to: "/list" }
            Route { to: "/list",       EntryScreen {}  }
            Route { to: "/detail", DetailScreen {}  }
        }
    )
}*/

pub fn CostEntryScreen(cx: Scope) -> Element {
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

#[derive(Deserialize)]
struct CostDetailScreenQuery {
    id: i32
}

pub fn CostDetailScreen(cx: Scope) -> Element {
    let route = use_route(cx);
    let router = use_router(cx);

    let id = match route.query::<CostDetailScreenQuery>() {
        None => { return render!("AHH BULLSHIT NO ID"); }
        Some(i) => { i }
    }.id;
    let http = use_context::<HTTP>(cx)?;

    let cost =
        use_future( cx, &(id,), move |(id,)| {
            get_cost(http.clone(), id)
        });
    let cost = cost.value()?.to_owned()?;
    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();
    let interpreted = interpret_cost(member.identity.id, &cost)?;

    let shares =
        use_future( cx, &(id,), move |(id,)| {
            get_shares(http.clone(), id)
        });
    let shares = shares.value()?.to_owned()?;

    let mut date = cost.added_on;
    #[cfg(feature = "web")]
    { // compile time conditional hook call is fine, because it's not runtime-conditional
        trace!("Trying to obtain timezone offset via eval");
        let eval = dioxus_web::use_eval(cx);
        let res = eval("let tz = new Date().getTimezoneOffset(); console.log('TZ: ',tz); return tz;").get();

        trace!("TZ res: {res:?}");

        if let Ok(serde_json::Value::Number(num)) = res {
            let off = time::UtcOffset::from_whole_seconds( (num.as_i64().unwrap_or(0) * -60) as i32 );
            if let Ok(off) = off {
                date = date.to_offset(off);
                trace!("TZ success!")
            } else {
                trace!("TZ failed to create offset");
            }
        } else {
            trace!("TZ failed: Not OK or not Number");
        }
    }
    #[cfg(not(feature = "web"))]
    {
        if let Ok(off) = time::UtcOffset::current_local_offset() {
            date = date.to_offset(off);
        }
    }
    let expanded_date = date.format(&format_description!("[weekday], der [day]. [month repr:long] [year], um [hour]:[minute]:[second] Uhr (GMT [offset_hour]:[offset_minute])")).expect("EE");

    // shares
    let share_obj = shares.iter().map(| share | {
        let usern = &member.friends[&share.debtor_id].name;
        let amt = if member.identity.id == share.debtor_id {-interpreted.single_payment} else {interpreted.single_payment};
        let strikethrough = share.paid || ( !interpreted.am_creditor &&  member.identity.id != share.debtor_id);

        rsx!(
            tr {
                td {
                    i {"{usern}"}
                    " übernimmt "
                }
                td {
                    AmountDisplay {
                        amt: amt,
                        strikethrough: strikethrough,
                    }
                    if share.paid {
                        rsx!(b { "✅ hat bezahlt" })
                    }
                }
            }
        )
    } );

    let verb = if interpreted.my_gain.is_sign_positive() { "bekomme zurück" } else { "zahle noch" };

    render!(
        HeaderBar { title: "Eintragsdetails für #{id} 🔎", }
        div {
            class: "scroll_container",

            CostEntry {
                c: cost
            }
            div {
                class: "cost_detail_card",

                "Datum: {expanded_date}"
            }
            table {
                share_obj
                hr {}
                tr {
                    td { "Ich {verb}:"  }
                    td { AmountDisplay { amt: interpreted.my_gain } }
                }
            }
        }
    )
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
                        amt: interpreted.my_gain,
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
pub fn AmountDisplay ( cx: Scope, amt: Decimal, strikethrough: Option<bool> ) -> Element {
    let strikethrough = strikethrough.unwrap_or(false);
    let mut amtstr = format!("{amt:.2}");
    if amt.is_sign_positive() {
        amtstr.insert(0, '+');
    }

    let class =
        if amt.is_zero() || strikethrough {
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