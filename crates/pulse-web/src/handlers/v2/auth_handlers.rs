use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::auth::jwt::create_token;
use crate::auth::password::{hash_password, verify_password};
use crate::auth::session::AuthUser;
use crate::db::models::PublicUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

fn err(code: &str, msg: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({"success": false, "error": {"code": code, "message": msg}})),
    )
}

fn unauthorized(code: &str, msg: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({"success": false, "error": {"code": code, "message": msg}})),
    )
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if body.email.is_empty() || !body.email.contains('@') {
        return Err(err(
            "VALIDATION_ERROR",
            "A valid email address is required.",
        ));
    }
    if body.username.len() < 3 || body.username.len() > 50 {
        return Err(err("VALIDATION_ERROR", "Username must be 3-50 characters."));
    }
    if body.password.len() < 8 {
        return Err(err(
            "VALIDATION_ERROR",
            "Password must be at least 8 characters.",
        ));
    }

    if let Ok(Some(_)) = state.repo.find_user_by_email(&body.email).await {
        return Err(err(
            "EMAIL_EXISTS",
            "An account with this email already exists.",
        ));
    }

    let password_hash = hash_password(&body.password).map_err(|_| {
        err(
            "SERVER_ERROR",
            "Unable to process registration. Please try again.",
        )
    })?;

    let user = state
        .repo
        .create_user(&body.email, &body.username, &password_hash)
        .await
        .map_err(|_| {
            err(
                "SERVER_ERROR",
                "Unable to create account. Please try again.",
            )
        })?;

    let token = create_token(&user.id.to_string(), &state.jwt_secret).map_err(|_| {
        err(
            "SERVER_ERROR",
            "Unable to create session. Please try again.",
        )
    })?;

    state
        .repo
        .create_session(user.id, &token)
        .await
        .map_err(|_| {
            err(
                "SERVER_ERROR",
                "Unable to create session. Please try again.",
            )
        })?;

    let public_user: PublicUser = user.into();

    Ok(Json(json!({
        "success": true,
        "data": {
            "token": token,
            "user": public_user
        }
    })))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user = state
        .repo
        .find_user_by_email(&body.email)
        .await
        .map_err(|_| err("SERVER_ERROR", "Unable to process login. Please try again."))?
        .ok_or_else(|| err("INVALID_CREDENTIALS", "Invalid email or password."))?;

    let valid = verify_password(&body.password, &user.password_hash).map_err(|_| {
        err(
            "SERVER_ERROR",
            "Unable to verify credentials. Please try again.",
        )
    })?;

    if !valid {
        return Err(unauthorized(
            "INVALID_CREDENTIALS",
            "Invalid email or password.",
        ));
    }

    let token = create_token(&user.id.to_string(), &state.jwt_secret).map_err(|_| {
        err(
            "SERVER_ERROR",
            "Unable to create session. Please try again.",
        )
    })?;

    state
        .repo
        .create_session(user.id, &token)
        .await
        .map_err(|_| {
            err(
                "SERVER_ERROR",
                "Unable to create session. Please try again.",
            )
        })?;

    let public_user: PublicUser = user.into();

    Ok(Json(json!({
        "success": true,
        "data": {
            "token": token,
            "user": public_user
        }
    })))
}

pub async fn logout(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let _ = state.repo.delete_all_sessions_for_user(auth.user_id).await;

    Ok(Json(json!({"success": true, "data": null})))
}

pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user = state
        .repo
        .find_user_by_id(auth.user_id)
        .await
        .map_err(|_| err("SERVER_ERROR", "Unable to load profile."))?
        .ok_or_else(|| err("NOT_FOUND", "User not found."))?;

    let public_user: PublicUser = user.into();

    Ok(Json(json!({
        "success": true,
        "data": public_user
    })))
}
