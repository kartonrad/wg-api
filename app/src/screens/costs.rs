#![allow(non_snake_case)]

use std::iter;
use std::ops::Sub;
use common::{Balance, BalancingTransaction, Cost, RegularDef, RegularSpending, UserDebt};
use dioxus::prelude::*;
use dioxus_router::{Link, use_route};
use log::trace;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::Deserialize;
use crate::{constants::API_URL, HeaderBar, use_api_else_return};
use time::macros::format_description;
use time::{Duration, Month};
use crate::identity_service::{upload_to_path, WGMember};
use rust_decimal_macros::dec;

pub fn CostListScreen(cx: Scope) -> Element {
    render!(
        CostList {
        }
    )
}


pub fn CostTallyScreen(cx: Scope) -> Element {
    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();

    let balances = use_api_else_return!(get_balances; cx);
    let balance_obj = balances.into_iter().map(|balance| {
        let _user = &member.friends[&balance.initiator_id];

        rsx!(
            BalanceEntry {
                b: balance
            }
        )
    });

    render!(
        Tallys {
        }
        //trx_obj
        h3 {
            class: "cost_seperator",
            "Vergangene Abrechnungen"
        }
        div {
            class: "scroll_container",
            balance_obj
        }

    )
}

fn pairwise<I>(right: I) -> impl Iterator<Item = (I::Item, I::Item)>
    where
        I: Iterator + Clone
{
    let left = right.clone();
    left.zip(right.skip(1))
}

pub fn CostStatScreen(cx: Scope) -> Element {
    let interval = RegularDef::Week;
    // algorithm expects these to be in descending order
    let stats = use_api_else_return!(get_stats; cx, interval);

    let n = 20usize;
    let mut now = crate::time::use_current_utc_time(cx).unwrap();
    let mut statpeeker = stats.into_iter().peekable();
    let mut stat_per_week = Vec::new();

    let mut y_max = dec!(0.0);

    while let Some(stat) = statpeeker.peek() {
        let year =  now.year();
        let week = now.iso_week();

        trace!("constructing: {now}, {year}, {week}");
        // if newer than 'now', discard (because what the hell???)
        if stat.time_bucket > now { let _ = statpeeker.next().unwrap(); /* guaranteed some() */ continue; }

        if stat.time_bucket.year() == now.year() && stat.time_bucket.iso_week() == now.iso_week() {
            let stat = statpeeker.next().unwrap();/* guaranteed some() */
            y_max = y_max.max( stat.total_unified_spending ).max( stat.i_paid ).max( stat.i_recieved ).max( stat.my_total_spending );

            stat_per_week.push(Some(stat))
        } else {
            // because we assume descending, we will walk back one week and try to match the same entry to that week
            stat_per_week.push(None);
        }
        now = now.sub(Duration::weeks(1));

        // we only want to display the last n elements
        if stat_per_week.len() > 20 { break; }
    }
    trace!("{y_max} {stat_per_week:?}");


    // SHIT DIAGRAM #1
    // more complicated for an uglier solution. constructing svg paths would be easier (in conclusion: bozo shit)
    // in pursuit of laziness i devised an insane contraption fckn pairwise iterators
    let line_iter = pairwise(stat_per_week.iter().rev()).enumerate()
        .map(|(idx, (from, to))| {
            let had_from = from.is_some();
            let from = from.clone().unwrap_or(RegularSpending::default());
            let to = to.clone().unwrap_or(RegularSpending::default());
            let date = from.time_bucket.format(&format_description!("[day] [month repr:short]")).expect("EE");

            trace!("from: {:?}, to: {:?}", from, to);

            rsx!(
                line {
                    stroke: "#80808059",
                    stroke_width: "1px",
                    x1: "{400/n*idx}",
                    x2: "{400/n*idx}",
                    y1: "{0}",
                    y2: "{300}",
                }

                line {
                    stroke: "#ff8f00c4",
                    x1: "{400/n*idx}",
                    x2: "{400/n*(idx+1)}",
                    y1: "{dec!(-300.0)*(from.total_unified_spending / y_max) +dec!(300.0)}",
                    y2: "{dec!(-300.0)*(to.total_unified_spending / y_max) +dec!(300.0)}",

                }
                line {
                    stroke: "blue",
                    x1: "{400/n*idx}",
                    x2: "{400/n*(idx+1)}",
                    y1: "{dec!(-300.0)*(from.my_total_spending / y_max) +dec!(300.0)}",
                    y2: "{dec!(-300.0)*(to.my_total_spending / y_max) +dec!(300.0)}",

                }
                line {
                    stroke: "#06bf008c",
                    x1: "{400/n*idx}",
                    x2: "{400/n*(idx+1)}",
                    y1: "{dec!(-300.0)*(from.i_recieved / y_max) +dec!(300.0)}",
                    y2: "{dec!(-300.0)*(to.i_recieved / y_max) +dec!(300.0)}",

                }
                line {
                    stroke: "#ff000080",
                    x1: "{400/n*idx}",
                    x2: "{400/n*(idx+1)}",
                    y1: "{dec!(-300.0)*(from.i_paid / y_max)+dec!(300.0)}",
                    y2: "{dec!(-300.0)*(to.i_paid / y_max)+dec!(300.0)}",

                }
                if had_from {
                    rsx!(
                        text {
                            writing_mode: "vertical-rl",
                            x: "{400/n*idx}",
                            y: "305",
                            color: "#80808059",
                            font_size: "10",

                            "{date}"
                        }
                    )
                }
             )
        })
        ;

    let grid_iter = (0..(y_max.trunc().to_i32().unwrap_or(0))).step_by( 10 ).map(|nr| {
        trace!("EEE {nr}");
        let y = dec!(300) - (Decimal::from(nr)/y_max*dec!(300));

        rsx!(
            line {
                stroke: "#80808059",
                stroke_width: "1px",
                x1: "0",
                x2: "400",
                y1: "{y}",
                y2: "{y}",
            }
        )
    });

    render!(
        svg {
            view_box: "0 0 400 350",
            xmlns: "http://www.w3.org/2000/svg",
            width: "100%",
            class: "weekely_stats",

            grid_iter
            line_iter
        }
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

    let cost = use_api_else_return!(get_cost; cx, id);

    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();
    let interpreted = interpret_cost(member.identity.id, &cost)?;

    let shares = use_api_else_return!(get_shares; cx, id);

    let mut date = cost.added_on;
    crate::time::use_date_to_local_offset(cx, &mut date);

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
        HeaderBar { title: "Eintrag #{id} 🔎", }
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
    let route = use_route(cx);

    let id = match route.query::<IdQuery>() {
        None => { return render!("AHH BULLSHIT NO ID"); }
        Some(i) => { i }
    }.id;

    render!(
        HeaderBar { title: "Abrechnung #{id} 🔎", }

        h3 {
            class: "cost_seperator",
            "Damaliger Stand"
        }

        Tallys {
            balance_id: id,
            include_trx: true
        }

        h3 {
            class: "cost_seperator",
            "Enthaltene Kosten"
        }

        CostList {
            balance_id: id
        }
    )
}

// ========== Components
#[inline_props]
fn TransactionEntry(cx: Scope, trx: BalancingTransaction) -> Element {
    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();

    let from_u = &member.friends[&trx.from_user_id];
    let to_u = &member.friends[&trx.to_user_id];

    let from_profile_pic = upload_to_path(from_u.profile_pic.clone()).unwrap_or("".to_string());
    let to_profile_pic = upload_to_path(to_u.profile_pic.clone()).unwrap_or("".to_string());

    render!(
        div {
            class: "transaction",

            div {
                class: "avatar",
                background_image: "url({API_URL}{from_profile_pic})",

                span {
                    "{from_u.name}"
                }
            }

            div {
                class: "transaction_arrow",

                AmountDisplay {
                    amt: trx.amt
                }
            }


            div {
                class: "avatar",
                background_image: "url({API_URL}{to_profile_pic})",

                span {
                    "{to_u.name}"
                }
            }
        }
    )
}

#[inline_props]
fn TallyTransactions(cx: Scope, tally: Vec<UserDebt>) -> Element {
    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();

    let trx = BalancingTransaction::from_debt_table(tally.clone())
        .expect("db return to be balancable as per shema");
    let trx_obj = trx
        .iter().map(| trx | {
        let from_u = &member.friends[&trx.from_user_id];
        let to_u = &member.friends[&trx.to_user_id];

        rsx!(
            TransactionEntry { trx: trx.to_owned() }
        )
    });

    render!(
        trx_obj
    )
}

#[inline_props]
fn Tallys(cx: Scope, balance_id: Option<i32>, include_trx: Option<bool>) -> Element {
    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();

    let balance_id = balance_id.to_owned();
    let tally = use_api_else_return!(get_tally; cx, balance_id);

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
                // these are *funny* metrics, but they have no practical value
                // and take up too much screen space
                // span { "Bekommt noch " AmountDisplay { amt: t.to_recieve } }br {}
                // span { "Und zahlt noch " AmountDisplay { amt: -t.to_pay } }br {}
                // hr {}
                span { "Das ergibt: " AmountDisplay { amt: t.to_recieve-t.to_pay } }
            }
        )
    });

    render!(
        tally_obj
        if include_trx.unwrap_or(false) {
            rsx!(
                h3 {
                    class: "cost_seperator",
                    "Ausgleichende Zahlungen"
                }
                TallyTransactions { tally: tally, }
            )
        } else { rsx!({}) }
    )
}


#[inline_props]
fn BalanceEntry( cx: Scope, b: Balance) -> Element {
    let member = use_shared_state::<WGMember>(cx).unwrap();
    let member = member.read();
    let user = &member.friends[&b.initiator_id];
    let profile_pic = upload_to_path( user.profile_pic.clone() ).unwrap_or("/public/img/rejection.jpg".to_string());


    "WG COST: {balance.total_unified_spending.unwrap_or(dec!(0.0))}, USER COST: {balance.my_total_spending.unwrap_or(dec!(0.0))}";

    let date = b.balanced_on.format(&format_description!("[weekday] [day] [month repr:short] [year]")).ok()?;

    render!(
        Link {
            to: "/costs/balance?id={b.id}",

            div {
                class: "cost_card",
                key: "{b.id}",

                div {
                    class: "body",

                    h4 { "Abrechnung: {date}" }

                    span {
                        div {
                            class: "avatar",
                            background_image: "url({API_URL}{profile_pic})",
                        }
                        "Angeordnet von " i {"{user.name}"}
                        hr {}
                        "Meine Ausgaben: " AmountDisplay {amt: b.my_total_spending?} br {}
                        "Ausgaben der WG:" AmountDisplay {amt: b.total_unified_spending?}
                    }
                }
                div {
                    class: "left",

                    "Bilanz:" br {}
                    AmountDisplay {
                        amt: b.i_recieved?-b.i_paid?,
                    }
                }
            }
        }
    )
}

#[inline_props]
fn CostList(cx:Scope, balance_id: Option<i32>) -> Element {
    let balance_id = balance_id.to_owned();
    let costs = use_api_else_return!(get_costs; cx, balance_id);

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
             Link {
                to : "/costs/detail?id={c.id}",
                class: "nolink",
                active_class: "disable_link",

                CostEntry {
                    c: c.clone(),
                }
            }
        ));

        last_week = week;
        last_month = month;
        last_year = year;
    });

    render!(
        div {
            class: "scroll_container",

            cost_obj.into_iter()
        }
    )
}

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