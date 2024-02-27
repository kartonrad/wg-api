use super::*;


#[derive(Clone, sqlx::FromRow)]
pub struct User {
    pub id : i32,
    pub username: String,

    pub name: String,
    pub bio: String,

    #[sqlx(json)]
    pub profile_pic: Option<Upload>,
    pub wg : Option<i32>,
}

impl User {
    pub async fn wg(&self, ctx: &AppContext) -> FieldResult<Option<WG>> {
        return if let Some(wg_id) = self.wg {
            let wg: WG = sqlx::query_as(r#"
                SELECT
                    wgs.id, wgs.url, wgs.name, wgs.description,
                    COALESCE(row_to_json(pp), 'null'::json) as profile_pic,
                    COALESCE(row_to_json(hp), 'null'::json) as header_pic FROM wgs
                LEFT JOIN uploads hp ON wgs.header_pic = hp.id
                LEFT JOIN uploads pp ON wgs.profile_pic = pp.id
                WHERE wgs.id = $1
            "#)
                .bind(wg_id)
                .fetch_one(&ctx.db_pool).await?;

            Ok(
                Some(wg)
            )
        } else {
            Ok(None)
        }
    }

}

#[graphql_object(context = AppContext)]
#[graphql(description = "A user of the app")]
impl User {
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn username(&self) -> &str { &self.username }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn bio(&self) -> &str {
        &self.bio
    }
    pub fn profile_pic(&self) -> Option<Upload> {
        self.profile_pic.clone()
    }
    pub fn wg_id(&self) -> Option<i32> {
        self.wg
    }

    pub fn is_me(&self, ctx: &AppContext) -> bool {
        Some(self.id) == ctx.authenticated_user.as_ref().map(|i| i.id)
    }
    pub async fn wg(&self, ctx: &AppContext) -> FieldResult<Option<WG>> {
        Self::wg(self, ctx).await
    }
}