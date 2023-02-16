
mod common;
use common::GLOBAL_TEST_STATE;
use reqwest::StatusCode;

#[actix_web::test]
pub async fn test_db_access() {
    let state = GLOBAL_TEST_STATE.get().await;
    let db = &state.pool;

    let eee: Option<i32> = sqlx::query_scalar!("SELECT 1+1 as result;").fetch_one(db).await.expect("Simple sql addition to succeed.");
    assert_eq!(eee.expect("1+1 to have a result"), 2);
}

#[actix_web::test]
pub async fn path_genesis() {
    let _state = GLOBAL_TEST_STATE.get().await;
    let req = reqwest::get("http://localhost:4269/").await.expect("Request to go through!")
        .status();
    assert_eq!(req, StatusCode::OK)
}