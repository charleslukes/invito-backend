use sqlx::*;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    model::UserModel,
    schema::{CreateUserSchema, FilterOptions, UpdateUserSchema},
    AppState,
};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Invito is running...";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn users_list_handler(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let query_result = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users ORDER by id LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Something bad happened while fetching all user items",
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }

    let users = query_result.unwrap();

    let json_response = serde_json::json!({
        "status": "success",
        "results": users.len(),
        "users": users
    });
    Ok(Json(json_response).into_response())
}

pub async fn create_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // checks if signup is with referral code
    if let Some(x) = body.ref_code {
        // check if code exits
        // increment user count for code owner
        let query_result = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE ref_code = $1", x)
            .fetch_one(&data.db)
            .await;

        match query_result {
            Ok(user) => {
                //update the ref user count
                let _ = sqlx::query_as!(
                    UserModel,
                    "UPDATE users SET added_by_ref_code = added_by_ref_code + 1 WHERE id = $1",
                    user.id
                )
                .fetch_one(&data.db);
            }
            Err(_) => {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("User with referral code: {} not found", x)
                });
                return Err((StatusCode::NOT_FOUND, Json(error_response)));
            }
        }
    }

    // creates new referral code
    let ref_id = Uuid::new_v4().to_string();
    let code = format!("{}{}", &body.user_name[0..3], &ref_id[0..4]);

    // add user to db
    let query_result = sqlx::query_as!(
        UserModel,
        "INSERT INTO users (email, user_name, ref_code, added_by_ref_code) VALUES ($1, $2, $3, $4) RETURNING *",
        body.email.to_string(),
        body.user_name.to_string(),
        code, 
        0
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(user) => {
            let user_response = json!({"status": "success",
                "message": "User created successfully",
                "data": json!({
                "user": user
            })});

            return Ok((StatusCode::CREATED, Json(user_response)));
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": "user with that email already exists",
                });
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    }
}

pub async fn get_user_handler(
    Path(user_name): Path<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users WHERE user_name = $1",
        user_name
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(user) => {
            let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "user": user
            })});

            return Ok((StatusCode::OK, Json(user_response)));
        }
        Err(_) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("{} not found", user_name)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
    }
}

pub async fn edit_user_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(UserModel, "SELECT * FROM users WHERE id = $1", id)
        .fetch_one(&data.db)
        .await;

    if query_result.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("User with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let now = chrono::Utc::now();
    let user = query_result.unwrap();

    let query_result = sqlx::query_as!(
        UserModel,
        "UPDATE users SET email = $1, user_name = $2, updated_at = $3 WHERE id = $4 RETURNING *",
        body.email.to_owned().unwrap_or(user.email),
        body.user_name.to_owned().unwrap_or(user.user_name),
        now,
        id
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(user) => {
            let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "user": user
            })});

            return Ok(Json(user_response));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", err)})),
            ));
        }
    }
}

pub async fn delete_user_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows_affected = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("User with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::NO_CONTENT)
}
