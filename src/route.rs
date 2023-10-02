use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handler::{
        create_user_handler, delete_user_handler, edit_user_handler, get_user_handler,
        health_checker_handler, users_list_handler,
    },
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route(
            "/api/users",
            get(users_list_handler).post(create_user_handler),
        )
        .route(
            "/api/user/:id",
            get(get_user_handler)
                .patch(edit_user_handler)
                .delete(delete_user_handler),
        )
        .with_state(app_state)
}
