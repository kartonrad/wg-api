use super::*;

#[derive(Clone, sqlx::FromRow, Deserialize)]
pub struct Upload {
    pub id: i32,
    pub extension: String,
    pub size_kb: i32,
    pub original_filename: String
}

#[graphql_object(context = AppContext)]
impl Upload {
    pub fn id(&self) -> i32 { self.id }
    pub fn extension(&self) -> String { self.extension.clone() }
    pub fn size_kb(&self) -> i32 { self.size_kb }
    pub fn original_filename(&self) -> String { self.original_filename.clone() }

    pub fn url(&self, ctx: &AppContext) -> String {
        format!("https://{}/uploads/{}.{}", ctx.server_origin, self.id, self.extension)
    }

}