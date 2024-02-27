use std::pin::Pin;
use juniper::{FieldError, FieldResult, graphql_object, RootNode};
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};
use juniper::futures::Stream;
use juniper::serde::Deserialize;
use crate::auth::Identity;

mod wg;
mod user;
mod upload;
mod equal_balance;
mod share;


pub use wg::*;
pub use user::*;
pub use upload::*;
pub use equal_balance::*;
pub use share::*;


#[derive(GraphQLObject)]
pub struct Share {
    int: i32,
}


pub struct QueryRoot;

impl QueryRoot {
    async fn me(ctx: &AppContext) -> FieldResult<User> {
        let identity = ctx.authenticated_user.clone().ok_or("Unauthenticated")?;

        let user : User = sqlx::query_as( r#"
            SELECT
                users.*,
                COALESCE(row_to_json(pp), 'null'::json) as profile_pic
            FROM users
            LEFT JOIN uploads pp ON users.profile_pic = pp.id
            WHERE users.id = $1
        "#)
            .bind(identity.id)
            .fetch_one(&ctx.db_pool).await?;

        return Ok(
            user
        );
    }
}

#[graphql_object(context = AppContext)]
impl QueryRoot {
    fn wg(_id: String) -> FieldResult<WG> {
        todo!();
    }

    async fn my_wg(ctx: &AppContext) -> FieldResult<Option<WG>> {
        let user: User = QueryRoot::me(ctx).await?;

        Ok(user.wg(ctx).await?)
    }

    async fn me(ctx: &AppContext) -> FieldResult<User> {
        QueryRoot::me(ctx).await
    }
}

pub struct MutationRoot;

#[graphql_object(context = AppContext)]
impl MutationRoot {
    fn cant_mutate() -> i32{
        todo!()
    }
}

pub struct Subscription;

#[juniper::graphql_subscription(context = AppContext)]
impl Subscription {
    async fn api_version() -> Pin<Box<dyn Stream<Item = Result<i32, FieldError>> + Send>> {
        todo!()
    }
}

pub struct AppContext {
    pub authenticated_user: Option<Identity>,
    pub db_pool: sqlx::PgPool,
    pub server_origin: String,
}

impl juniper::Context for AppContext {}

pub type Schema = RootNode<'static, QueryRoot, MutationRoot, Subscription>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot {}, MutationRoot {}, Subscription {})
}