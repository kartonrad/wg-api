#![allow(non_snake_case)]

use crate::api::{get_balances, get_cost, get_costs, get_shares, get_stats, get_tally, HTTP};
use crate::identity_service::{upload_to_path, WGMember};
use crate::time::{current_utc_time, date_to_local_offset, local_tz_offset};
use crate::Route;
use crate::{constants::API_URL, use_api_else_return, HeaderBar};
use common::{
    Balance, BalancingTransaction, Cost, CostInput, RegularDef, RegularSpending, UserDebt,
};
use dioxus::core::anyhow;
use dioxus::prelude::*;
use dioxus_router::{use_route, use_router, Link};
use futures_lite::io::split;
use futures_lite::FutureExt;
use log::{error, trace};
use reqwest::header::CONTENT_TYPE;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Sub;
use time::format_description::well_known::Iso8601;
use time::{format_description, Duration, Month, OffsetDateTime, PrimitiveDateTime};

#[component]
pub fn CostNewScreen() -> Element {
    let http = use_context::<HTTP>();
    let mmember = use_context::<Signal<WGMember>>();
    let member = mmember.read();
    let router = use_router();

    // DATE
    let time = use_resource(|| {
        Box::pin(async {
            let utc_offset = local_tz_offset().await?;
            let current_date = current_utc_time().await?;
            let current_date = date_to_local_offset(current_date).await;

            Some((utc_offset, current_date))
        })
    });
    let Some(Some((utc_offset, current_date))) = time.read_unchecked().cloned() else {
        return rsx!("loading time...");
    };

    let mut added_on = use_signal(|| current_date);
    let added_on_str = added_on.read().format(
        // TODO: in konstante auslagern?
        &format_description::parse("[year]-[month]-[day]T[hour]:[minute]")
            .expect("Valid Format Description"),
    )?;
    trace!("ISO Current date: {added_on_str}");

    // AMOUNT
    let mut decimal_str = use_signal(|| "0.00".to_string());
    let decimal = Decimal::from_str_exact(&decimal_str.read()).unwrap_or(dec!(0.0));

    let on_amount_change = move |evt: Event<FormData>| {
        trace!("WHAT THE HELL");
        let mut amt = evt.value().clone();

        // Checks if user typed some bullshit character
        let is_numeric = amt.chars().fold(true, |bool, char| {
            return bool && (char.is_ascii_digit() || char == '.');
        });

        if is_numeric {
            amt = amt.replace(|c: char| !char::is_ascii_digit(&c), "");
            amt = format!("{amt:0>3}");

            trace!("STEP 1 => {amt}");

            let int_part = &amt[0..amt.len() - 2];
            let fract_part = &amt[amt.len() - 2..];
            amt = format!("{int_part}.{fract_part}");

            trace!("STEP 2 => {amt}");

            amt = amt.replace("0", " ").trim_start().replace(" ", "0");
            amt = format!("{amt:0>4}");

            trace!("STEP 3 => {amt}");

            decimal_str.set(amt.clone());
        } else {
            trace!("Ignored fool");
            decimal_str.set(amt.clone());
        }
    };

    // DEBTORS
    let mut debtors = use_signal(|| {
        let debtors: HashMap<i32, bool> = member
            .friends
            .iter()
            .map(|(t, user)| (t.clone(), true))
            .collect();
        debtors
    });

    let debtors_selectors = member.friends.clone().into_iter().map(|(t, user)| {
        let profile_pic = upload_to_path(user.profile_pic.clone())
            .unwrap_or("/public/img/rejection.jpg".to_string());
        let active = *debtors.read().get(&t).unwrap_or(&false);

        rsx!(
            div {
                class: "new_cost_debtor",
                "data-active": "{active}",
                onclick: move |evt| { let _ = debtors.write().insert(t, !active); },

                div {
                    class: "avatar",
                    background_image: "url({API_URL}{profile_pic})",
                }

                h2 { "{user.name}" }
            }
        )
    });

    // CURRENT STATE ANALYSOS
    let debtor_list: Vec<i32> = debtors
        .read()
        .iter()
        .filter(|(t, b)| **b)
        .map(|(t, b)| t.clone())
        .collect();
    let debtor_count = debtor_list.len();
    let debtor_clearcount = debtor_list
        .iter()
        .filter(|t| member.identity.id != **t)
        .count();

    // ON SUBMIT

    let send_cost = move |evt: FormEvent| {
        evt.stop_propagation();
        evt.prevent_default();
        let me_id = mmember.read().identity.id;
        spawn({
            to_owned![router, http, decimal, debtor_list];
            let added_on = added_on.cloned();

            let form_values = evt.values();
            let name = form_values.iter().find(|entry| entry.0 == "name");
            let name = match name {
                Some((_, FormValue::Text(name))) => name.to_owned(),
                _ => "Unnamed >:(".to_string(),
            };

            async move {
                to_owned![added_on, name];

                let req = http
                    .post(format!("{API_URL}/api/my_wg/costs"))
                    .header(CONTENT_TYPE, "application/json")
                    .body(
                        serde_json::to_string(&CostInput {
                            // TODO: Add ability to submit for someone else
                            on_behalf_of_user_id: None,
                            name: name.to_owned(),
                            amount: decimal,
                            added_on: added_on,
                            debtors: debtor_list.into_iter().map(|i| (i, i == me_id)).collect(),
                        })
                        .unwrap_or("inmvalid json ad".to_owned()),
                    );

                let result = req.send().await;
                match result.map(|res| res.error_for_status()) {
                    Ok(res) => {
                        trace!("SUCCESSFULLY ADDED ENTRY!");
                        router.go_back();
                    }
                    Err(err) => {
                        error!("Couldn't add entry!! {err}");
                    }
                }
            }
        });
        //evt.values
    };

    rsx!(
        HeaderBar { title: "Eintrag anlegen", }

        form {
            onsubmit: send_cost,
            // prevent_default: "onsubmit",

            div {
                background_color: "rgba(255, 255, 255, 0.35)",
                //style: "backdrop-filter: blur(2px);",s

                div {
                    class: "description_and_value_input",

                    textarea {
                        name: "name",
                        placeholder: "Beschreibung",
                        rows: 3,
                    }

                    input {
                        r#type: "text",
                        name: "amount",
                        value: "{decimal_str}",
                        oninput: on_amount_change,
                    }
                }

                h5 { class: "cost_seperator", "Datum/Uhrzeit:" }

                input {
                    r#type: "datetime-local",
                    name: "added_on",
                    value: "{added_on_str}",
                    oninput: move |evt: Event<FormData>| {
                        let ee = &evt.value();
                        trace!("Trying to parse: {ee}");
                        added_on.set(
                            match PrimitiveDateTime::parse(&evt.value().trim(),
                                &format_description::parse("[year]-[month]-[day]T[hour]:[minute]")
                                    .expect("Valid Format Description"),
                            ) {
                                Ok(date) => date.assume_offset(utc_offset),
                                Err(err) => {
                                    trace!("{err}");
                                    let val = &evt.value();
                                    trace!("PArsing: {val} - failed!");
                                    current_date
                                }
                            }
                        )
                    }
                }

                h5 { class: "cost_seperator", "Beteiligte:"}

                div {
                    class: "new_cost_debtors_container",
                    { debtors_selectors }
                }


            }

            div {
                class: "wg_body", // misuse

                if debtor_count>0 {
                    "Du hast {decimal_str}€ bezahlt"
                    br {}
                    "Du bekommst"
                    AmountDisplay {
                        amt: decimal / Decimal::from(debtor_count) * Decimal::from(debtor_clearcount)
                    }
                    "von den {debtor_count} anderen Beteiligten zurück."
                } else {
                    span {
                        color: "crimson", "Wähle mindestens eine/n Beteiligte/n aus!"
                    }
                }
            }

            input {
                r#type: "submit",
                value: "Eintrag anlegen!"
            }
        }
    )
}

#[component]
pub fn CostListScreen() -> Element {
    rsx!(
        CostList {
        }
        Link {
            to: "/costs/new",
            class: "floating_new_button",

            span { "➕" }
        }
    )
}

#[component]
pub fn CostTallyScreen() -> Element {
    let member = use_context::<Signal<WGMember>>();
    let member = member.read();

    let client = use_context::<HTTP>();
    let val = use_resource(move || get_balances(client.clone()));
    let Some(balances) = val.cloned() else {
        return rsx!("loading!");
    };
    let balances = balances.context("Balances could not be retrieved")?;

    let balance_obj = balances.into_iter().map(|balance| {
        let _user = &member.friends[&balance.initiator_id];

        rsx!(BalanceEntry { b: balance })
    });

    rsx!(
        Tallys {
        }
        //trx_obj
        h3 {
            class: "cost_seperator",
            "Vergangene Abrechnungen"
        }
        div {
            class: "scroll_container",
            { balance_obj }
        }

    )
}

fn pairwise<I>(right: I) -> impl Iterator<Item = (I::Item, I::Item)>
where
    I: Iterator + Clone,
{
    let left = right.clone();
    left.zip(right.skip(1))
}

#[component]
pub fn CostStatScreen() -> Element {
    let interval = RegularDef::Week;
    let interval2 = interval.clone();
    // algorithm expects these to be in descending order
    //let stats = use_api_else_return!(get_stats; interval);

    use dioxus::core::AnyhowContext;

    let client = use_context::<HTTP>();
    let val = use_resource(move || get_stats(client.clone(), interval2.clone()));
    let Some(stats) = val.cloned() else {
        return rsx!("loading!");
    };
    let stats = stats.context("Stats could not be retrieved")?;

    let n = 20usize;

    let time = use_resource(|| Box::pin(async { current_utc_time().await }));
    let Some(Some(mut now)) = time.read_unchecked().cloned() else {
        return rsx!("loading time");
    };

    let mut statpeeker = stats.into_iter().peekable();
    let mut stat_per_week = Vec::new();

    let mut y_max = dec!(0.0);

    while let Some(stat) = statpeeker.peek() {
        let year = now.year();
        let week = now.iso_week();

        trace!("constructing: {now}, {year}, {week}");
        // if newer than 'now', discard (because what the hell???)
        if stat.time_bucket > now {
            let _ = statpeeker.next().unwrap(); /* guaranteed some() */
            continue;
        }

        if stat.time_bucket.year() == now.year() && stat.time_bucket.iso_week() == now.iso_week() {
            let stat = statpeeker.next().unwrap(); /* guaranteed some() */
            y_max = y_max
                .max(stat.total_unified_spending)
                .max(stat.i_paid)
                .max(stat.i_recieved)
                .max(stat.my_total_spending);

            stat_per_week.push(Some(stat))
        } else {
            // because we assume descending, we will walk back one week and try to match the same entry to that week
            stat_per_week.push(None);
        }
        now = now.sub(Duration::weeks(1));

        // we only want to display the last n elements
        if stat_per_week.len() > 20 {
            break;
        }
    }
    trace!("{y_max} {stat_per_week:?}");

    // SHIT DIAGRAM #1
    // more complicated for an uglier solution. constructing svg paths would be easier (in conclusion: bozo shit)
    // in pursuit of laziness i devised an insane contraption fckn pairwise iterators
    let line_iter = pairwise(stat_per_week.iter().rev())
        .enumerate()
        .map(|(idx, (from, to))| {
            let had_from = from.is_some();
            let from = from.clone().unwrap_or(RegularSpending::default());
            let to = to.clone().unwrap_or(RegularSpending::default());
            let date = from
                .time_bucket
                .format(
                    &format_description::parse("[day] [month repr:short]")
                        .expect("Valid format description"),
                )
                .expect("EE");

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
                   text {
                       writing_mode: "vertical-rl",
                       x: "{400/n*idx}",
                       y: "305",
                       color: "#80808059",
                       font_size: "10",

                       "{date}"
                   }
               }
            )
        });

    let grid_iter = (0..(y_max.trunc().to_i32().unwrap_or(0)))
        .step_by(10)
        .map(|nr| {
            trace!("EEE {nr}");
            let y = dec!(300) - (Decimal::from(nr) / y_max * dec!(300));

            rsx!(line {
                stroke: "#80808059",
                stroke_width: "1px",
                x1: "0",
                x2: "400",
                y1: "{y}",
                y2: "{y}",
            })
        });

    rsx!(
        svg {
            view_box: "0 0 400 350",
            xmlns: "http://www.w3.org/2000/svg",
            width: "100%",
            class: "weekely_stats",

            { grid_iter }
            { line_iter }
        }
    )
}

#[derive(Deserialize)]
struct IdQuery {
    id: i32,
}

#[component]
pub fn CostDetailScreen(cost_id: i32) -> Element {
    let client = use_context::<HTTP>();
    let val = use_resource(move || get_cost(client.clone(), cost_id));
    let Some(cost) = val.cloned() else {
        return rsx!("loading!");
    };
    let cost = cost.context("Cost could not be retrieved")?;

    let member = use_context::<Signal<WGMember>>();
    let member = member.read();
    let interpreted =
        interpret_cost(member.identity.id, &cost).ok_or(anyhow!("Failed to interpret cost"))?;

    //let shares = use_api_else_return!(get_shares; cx, id);
    let client = use_context::<HTTP>();
    let val = use_resource(move || get_shares(client.clone(), cost_id));
    let Some(shares) = val.cloned() else {
        return rsx!("loading!");
    };
    let shares = shares.context("Shares could not be retrieved")?;

    let date = cost.added_on.clone();
    let date = use_resource(move || Box::pin(async move { date_to_local_offset(date).await }));
    let Some(date) = date.cloned() else {
        return rsx!("loading time!");
    };

    let expanded_date = date.format(
        &format_description::parse("[weekday], der [day]. [month repr:long] [year],\n um [hour]:[minute]:[second] Uhr (GMT [offset_hour]:[offset_minute])").unwrap()
    ).expect("EE");

    // shares
    let share_obj = shares.iter().map(|share| {
        let usern = &member.friends[&share.debtor_id].name;
        let amt = if member.identity.id == share.debtor_id {
            -interpreted.single_payment
        } else {
            interpreted.single_payment
        };
        let strikethrough =
            share.paid || (!interpreted.am_creditor && member.identity.id != share.debtor_id);

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
                        b { "✅" }
                    }
                }
            }
        )
    });

    let verb = if interpreted.my_gain.is_sign_positive() {
        "bekomme zurück"
    } else {
        "zahle noch"
    };

    rsx!(
        HeaderBar { title: "Eintrag #{cost_id} 🔎", }
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

                { share_obj }
                hr {}
                tr {
                    td { "Ich {verb}:"  }
                    td { AmountDisplay { amt: interpreted.my_gain } }
                }
            }
        }
    )
}

#[component]
pub fn CostBalanceDetailScreen(balance_id: i32) -> Element {
    rsx!(
        HeaderBar { title: "Abrechnung #{balance_id} 🔎", }

        h3 {
            class: "cost_seperator",
            "Damaliger Stand"
        }

        Tallys {
            balance_id,
            include_trx: true
        }

        h3 {
            class: "cost_seperator",
            "Enthaltene Kosten"
        }

        CostList {
            balance_id
        }
    )
}

// ========== Components
#[component]
fn TransactionEntry(trx: BalancingTransaction) -> Element {
    let member = use_context::<Signal<WGMember>>();
    let member = member.read();

    let from_u = &member.friends[&trx.from_user_id];
    let to_u = &member.friends[&trx.to_user_id];

    let from_profile_pic = upload_to_path(from_u.profile_pic.clone()).unwrap_or("".to_string());
    let to_profile_pic = upload_to_path(to_u.profile_pic.clone()).unwrap_or("".to_string());

    rsx!(
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

#[component]
fn TallyTransactions(tally: Vec<UserDebt>) -> Element {
    let trx = BalancingTransaction::from_debt_table(tally.clone())
        .expect("db return to be balancable as per shema");
    let trx_obj = trx.iter().map(|trx| {
        rsx!(TransactionEntry {
            trx: trx.to_owned()
        })
    });

    rsx!({ trx_obj })
}

#[component]
fn Tallys(balance_id: Option<i32>, include_trx: Option<bool>) -> Element {
    let member = use_context::<Signal<WGMember>>();
    let member = member.read();

    let balance_id = balance_id.to_owned();

    let client = use_context::<HTTP>();
    let val = use_resource(move || get_tally(client.clone(), balance_id));
    let Some(tally) = val.cloned() else {
        return rsx!("Loading...");
    };
    let tally = tally.context("Balance could not be retrieved")?;

    let tally_obj = tally.iter().map(|t| {
        let user = &member.friends[&t.user_id];
        let profile_pic = upload_to_path(user.profile_pic.clone())
            .unwrap_or("/public/img/rejection.jpg".to_string());

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

    rsx!(
        { tally_obj }
        if include_trx.unwrap_or(false) {
            { rsx!(
                    h3 {
                        class: "cost_seperator",
                        "Ausgleichende Zahlungen"
                    }
                    TallyTransactions { tally: tally, }
            ) }
        } else {
        }
    )
}

#[component]
fn BalanceEntry(b: Balance) -> Element {
    let member = use_context::<Signal<WGMember>>();
    let member = member.read();

    let user = &member.friends[&b.initiator_id];
    let profile_pic =
        upload_to_path(user.profile_pic.clone()).unwrap_or("/public/img/rejection.jpg".to_string());

    "WG COST: {balance.total_unified_spending.unwrap_or(dec!(0.0))}, USER COST: {balance.my_total_spending.unwrap_or(dec!(0.0))}";

    let date = b.balanced_on.format(
        &format_description::parse("[weekday] [day] [month repr:short] [year]")
            .expect("Format description valid!"),
    )?;

    let Balance {
        id,
        balanced_on,
        initiator_id,
        wg_id,
        total_unified_spending,
        i_paid,
        i_recieved,
        my_total_spending,
    } = b;
    let my_total_spending = my_total_spending.ok_or(anyhow!("This value is required"))?;
    let total_unified_spending = total_unified_spending.ok_or(anyhow!("This value is required"))?;
    let i_recieved = i_recieved.ok_or(anyhow!("This value is required"))?;
    let i_paid = i_paid.ok_or(anyhow!("This value is required"))?;

    rsx!(
        Link {
            to: "/costs/balance?balance_id={b.id}",

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
                        "Meine Ausgaben: " AmountDisplay {amt: my_total_spending} br {}
                        "Ausgaben der WG:" AmountDisplay {amt: total_unified_spending}
                    }
                }
                div {
                    class: "left",

                    "Bilanz:" br {}
                    AmountDisplay {
                        amt: i_recieved-i_paid,
                    }
                }
            }
        }
    )
}

#[component]
fn CostList(balance_id: Option<i32>) -> Element {
    let balance_id = balance_id.to_owned();
    //let costs = use_api_else_return!(get_costs; cx, balance_id);

    let client = use_context::<HTTP>();
    let val = use_resource(move || get_costs(client.clone(), balance_id));
    // TODO: Error Displaying?
    let Some(costs) = val.read().clone() else {
        return rsx!("Loading...");
    };
    let costs = costs.context("Costs could not be retrieved")?;

    let mut cost_obj: Vec<Element> = vec![];

    let Some(first_cost) = costs.get(0) else {
        // Empty
        return rsx!(div {
            class: "scroll_container",
        });
    };

    let mut last_year: i32 = first_cost.added_on.year();
    let mut last_month: Month = first_cost.added_on.month();
    let mut last_week: u8 = first_cost.added_on.iso_week();

    costs.iter().for_each(|c| {
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
                to : "/costs/detail?cost_id={c.id}",
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

    rsx!(
        div {
            class: "scroll_container",

            { cost_obj.into_iter() }
        }
    )
}

#[component]
fn CostEntry(c: Cost) -> Element {
    let member = use_context::<Signal<WGMember>>();
    let member = member.read();
    let interpreted = interpret_cost(member.identity.id, &c)
        .ok_or(anyhow!("Could not interpret the cost data."))?;

    let user = &member.friends[&c.creditor_id];
    let profile_pic = upload_to_path(user.profile_pic.clone()).unwrap_or("".to_string());

    let amt = c.amount.round_dp(2);
    let date = c
        .added_on
        .format(
            &format_description::parse("[day] [month repr:short]")
                .expect("Format description should be fine."),
        )
        .expect("EE");

    rsx!(
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
    am_creditor: bool,
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
        my_gain -= if my_share_paid {
            Decimal::ZERO
        } else {
            single_payment
        };
    }

    return Some(InterpretedCost {
        my_gain,
        single_payment,
        am_creditor,
    });
}

#[component]
pub fn AmountDisplay(amt: Decimal, strikethrough: Option<bool>) -> Element {
    let strikethrough = strikethrough.unwrap_or(false);
    let mut amtstr = format!("{amt:.2}");
    if amt.is_sign_positive() {
        amtstr.insert(0, '+');
    }

    let class = if amt.is_zero() || strikethrough {
        "amount_display zero"
    } else {
        if amt.is_sign_positive() {
            "amount_display positive"
        } else {
            "amount_display negative"
        }
    };

    rsx!(
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
