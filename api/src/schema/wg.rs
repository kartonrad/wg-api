use super::*;

#[derive(sqlx::FromRow, Deserialize, Clone)]
pub struct WG {
    pub id : i32,
    pub url: String,

    pub name: String,
    pub description: String,
    #[sqlx(json)]
    pub profile_pic: Option<Upload>,
    #[sqlx(json)]
    pub header_pic: Option<Upload>
}

#[graphql_object(context = AppContext)]
#[graphql(description = "A WG")]
impl WG {
    fn id(&self) -> i32 {
        self.id
    }
    fn url(&self) -> &str {
        &self.url
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        &self.description
    }
    fn profile_pic(&self) -> Option<Upload> {
        self.profile_pic.clone()
    }
    fn header_pic(&self) -> Option<Upload> {
        self.header_pic.clone()
    }

    async fn users(&self, ctx: &AppContext) -> FieldResult<Vec<User>> {
        let users : Vec<User> = sqlx::query_as( r#"
            SELECT
                users.*,
                COALESCE(row_to_json(pp), 'null'::json) as profile_pic
                FROM users
            LEFT JOIN uploads pp ON users.profile_pic = pp.id
            WHERE users.wg = $1
        "#)
            .bind(self.id)
            .fetch_all(&ctx.db_pool).await?;

        return Ok(
            users
        );
    }

    fn equal_balances() -> Vec<EqualBalance> {
        todo!()
    }
    fn costs(&self, ctx: &AppContext) -> Vec<Share> {



        todo!()
    }
}