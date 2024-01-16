use std::net::TcpListener;

use axum::{extract::{State, Path}, http::StatusCode, response::Json};

use diesel::prelude::*;
// use std::net::SocketAddr;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    model::*,
    schema::{tickets, users},
};

pub async fn create_user(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<User>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| {
            diesel::insert_into(users::table)
                .values(new_user)
                .returning(User::as_returning())
                .get_result(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

pub async fn list_users(
    State(pool): State<deadpool_diesel::postgres::Pool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| users::table.select(User::as_select()).load(conn))
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

// list tickets
pub async fn list_tickets(
    State(pool): State<deadpool_diesel::postgres::Pool>,
) -> Result<Json<Vec<Ticket>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(|conn| tickets::table.select(Ticket::as_select()).load(conn))
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

// list tickets by userid
pub async fn list_tickets_by_userid(
    Path(uid): Path<i64>,
    State(pool): State<deadpool_diesel::postgres::Pool>,
    // Json(new_user): Json<NewUser>,
) -> Result<Json<Vec<Ticket>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    // let uid = query_user_id;
    let res = conn
        .interact(move |conn | {
            use crate::schema::tickets::dsl::*;
            tickets.filter(user_id.eq(uid)).load(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;
    Ok(Json(res))
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
