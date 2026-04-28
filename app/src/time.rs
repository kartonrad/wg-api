use dioxus::prelude::*;
use log::trace;
use time::{OffsetDateTime, UtcOffset};

pub async fn date_to_local_offset(mut date: OffsetDateTime) -> OffsetDateTime {
    #[cfg(feature = "web")]
    {
        // compile time conditional hook call is fine, because it's not runtime-conditional

        trace!("Trying to obtain timezone offset via eval");

        let res = document::eval(
            "let tz = new Date().getTimezoneOffset(); console.log('TZ: ',tz); return tz;",
        )
        .await;

        trace!("TZ res: {res:?}");

        if let Ok(serde_json::Value::Number(num)) = res {
            let off = time::UtcOffset::from_whole_seconds((num.as_i64().unwrap_or(0) * -60) as i32);
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
            *date = date.to_offset(off);
        }
    }

    date
}

pub async fn local_tz_offset() -> Option<UtcOffset> {
    #[cfg(feature = "web")]
    {
        // compile time conditional hook call is fine, because it's not runtime-conditional
        trace!("Trying to obtain timezone offset via eval");
        let res = document::eval(
            "let tz = new Date().getTimezoneOffset(); console.log('TZ: ',tz); return tz;",
        )
        .await;

        trace!("TZ res: {res:?}");

        if let Ok(serde_json::Value::Number(num)) = res {
            let off = time::UtcOffset::from_whole_seconds((num.as_i64().unwrap_or(0) * -60) as i32);
            if let Ok(off) = off {
                trace!("TZ success!");
                return Some(off);
            } else {
                trace!("TZ failed to create offset");
            }
        } else {
            trace!("TZ failed: Not OK or not Number");
        }
        None
    }
    #[cfg(not(feature = "web"))]
    {
        time::UtcOffset::current_local_offset()
    }
}

pub async fn current_utc_time() -> Option<OffsetDateTime> {
    #[cfg(feature = "web")]
    {
        // compile time conditional hook call is fine, because it's not runtime-conditional
        trace!("Trying to obtain current time  via eval");
        let res =
            document::eval("let tz = new Date().getTime(); console.log('Time: ',tz); return tz;")
                .await;

        trace!("Time res: {res:?}");

        if let Ok(serde_json::Value::Number(num)) = res {
            let off = OffsetDateTime::from_unix_timestamp(num.as_i64().unwrap_or(0) / 1000);
            if let Ok(off) = off {
                trace!("Time success! {}", off);
                Some(off)
            } else {
                trace!("Time failed to create offsetdatetime");
                None
            }
        } else {
            trace!("Time failed: Not OK or not Number");
            None
        }
    }
    #[cfg(not(feature = "web"))]
    {
        Some(time::OffsetDateTime::now_utc())
    }
}

//#[cfg(feature = "desktop")]
//use dioxus::EvalResult;
//#[cfg(feature = "web")]
//use dioxus_web::EvalResult;

/*
pub fn get_current_utc_time(  eval:  &dyn Fn(S) -> EvalResult ) -> Option<OffsetDateTime> {

}*/
