use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserSchema {
    pub user_name: String,
    pub email: String,
    pub ref_code: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserSchema {
    pub user_name: Option<String>,
    pub email: Option<String>,
}
