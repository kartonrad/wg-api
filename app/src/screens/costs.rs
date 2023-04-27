use common::{BalancingTransaction, Cost};
use dioxus::prelude::*;
use dioxus_router::{Link, use_route};
use log::trace;
use rust_decimal::Decimal;
use serde::Deserialize;
use crate::{api::{HTTP, upload_to_path, WGMember}, api, constants::API_URL, HeaderBar, TopTabs, use_api_else_return};
use time::macros::format_description;
use time::Month;

pub fn CostListScreen(cx: Scope) -> Element {
    let http = use_context::<HTTP>(cx)?;

    let costs = use_api_else_return!(get_costs; cx, http);

    let mut cost_obj: Vec<LazyNodes> = vec![];
    let mut last_year: i32 = costs.get(0)?.added_on.year();
    let mut last_month: Month = costs.get(0)?.added_on.month();
    let mut last_week: u8 = costs.get(0)?.added_on.iso_week();

    costs.iter().for_each(|c|{
        let year: i32 = c.added_on.year();
        let month: Month = c.added_on.month();
        let week: u8 = c.added_on.iso_week();

        if week != last_week {
            cost_obj.push(rsx!(
                h3 {
                    class: "cost_seperator",
                    "KW {week}"
                }
            ))
        }
        if month != last_month {
            cost_obj.push(rsx!(
                h2 {
                    class: "cost_seperator",
                    "{month}"
                }
            ))
        }
        if year != last_year {
            cost_obj.push(rsx!(
                h1 {
                    class: "cost_seperator",
                    "{year}"
                }
            ))
        }

        cost_obj.push(rsx!(
            CostEntry {
                c: c.clone(),
            }
        ));

        last_week = week;
        last_month = month;
        last_year = year;
    });

    render!(
        TopTabs {}
        div {
            class: "scroll_container",

            cost_obj.into_iter()
        }
    )
}


pub fn CostTallyScreen(cx: Scope) -> Element {
    let http = use_context::<HTTP>(cx)?;
    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();

    let tally_balance_id = None;
    let tally = use_api_else_return!(get_tally; cx, http, tally_balance_id);

    let trx = BalancingTransaction::from_debt_table(tally.clone())
        .expect("db return to be balancable as per shema");
    let trx_obj = trx
        .iter().map(| trx | {
        let from_u = &member.friends[&trx.from_user_id];
        let to_u = &member.friends[&trx.to_user_id];

        rsx!(div {
                "{from_u.name} zahlt {trx.amt} an {to_u.name}!"
            })
    });

    let tally_obj = tally.iter().map(|t| {
        let user = &member.friends[&t.user_id];
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
                span { "Bekommt noch " AmountDisplay { amt: t.to_recieve } }br {}
                span { "Und zahlt noch " AmountDisplay { amt: -t.to_pay } }br {}
                hr {}
                span { "Das ergibt: " AmountDisplay { amt: t.to_recieve-t.to_pay } }
            }
        )
    });



    render!(
        TopTabs {}
        tally_obj
        trx_obj
    )
}

pub fn CostStatScreen(cx: Scope) -> Element {
    render!(
        TopTabs {}
        "Stat"
    )
}


#[derive(Deserialize)]
struct IdQuery {
    id: i32
}

pub fn CostDetailScreen(cx: Scope) -> Element {
    let route = use_route(cx);

    let id = match route.query::<IdQuery>() {
        None => { return render!("AHH BULLSHIT NO ID"); }
        Some(i) => { i }
    }.id;
    let http = use_context::<HTTP>(cx)?;

    let cost = use_api_else_return!(get_cost; cx, http, id);

    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();
    let interpreted = interpret_cost(member.identity.id, &cost)?;

    let shares = use_api_else_return!(get_shares; cx, http, id);

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
    let expanded_date = date.format(&format_description!("[weekday], der [day]. [month repr:long] [year],\n um [hour]:[minute]:[second] Uhr (GMT [offset_hour]:[offset_minute])")).expect("EE");

    // shares
    let share_obj = shares.iter().map(| share | {
        let usern = &member.friends[&share.debtor_id].name;
        let amt = if member.identity.id == share.debtor_id {-interpreted.single_payment} else {interpreted.single_payment};
        let strikethrough = share.paid || ( !interpreted.am_creditor &&  member.identity.id != share.debtor_id);

        rsx!(
            tr {
                td {
                    i {"{usern}"}
                    if share.paid {" übernahm "} else { " übernimmt " }
                }
                td {
                    AmountDisplay {
                        amt: amt,
                        strikethrough: strikethrough,
                    }
                    if share.paid {
                        rsx!(b { "✅" })
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
                class: "cost_detail_date",
                white_space: "pre",

                "{expanded_date}"
            }
            table {
                class: "cost_detail_calculation",

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


pub fn CostBalanceDetailScreen(cx: Scope) -> Element {
    render!("BalanceDetail")
}


// Cost Object
#[inline_props]
fn CostEntry(cx: Scope, c: Cost) -> Element {
    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();
    let interpreted = interpret_cost(member.identity.id, &c)?;

    let user = &member.friends[&c.creditor_id];
    let profile_pic = upload_to_path(user.profile_pic.clone()).unwrap_or("".to_string());


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

    return Some(InterpretedCost {my_gain, single_payment, am_creditor});
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