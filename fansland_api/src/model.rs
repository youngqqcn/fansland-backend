use diesel::prelude::*;

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Text,
        hair_color -> Nullable<Text>,
    }
}

#[derive(serde::Serialize, Selectable, Queryable)]
pub struct User {
    id: i32,
    name: String,
    hair_color: Option<String>,
}

#[derive(serde::Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    name: String,
    hair_color: Option<String>,
}
