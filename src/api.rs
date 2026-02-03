use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::sync::Arc;

use crate::auth::{auth_middleware, AuthState};
use crate::notify::{NotifyRequest, Notifier};
use crate::scheduler::{NotificationScheduler, OneTimeRequest, ScheduleRequest};

#[derive(Clone)]
pub struct AppState {
    pub notifier: Arc<Notifier>,
    pub scheduler: Arc<NotificationScheduler>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct NotifyResponse {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct JobCreatedResponse {
    pub job_id: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

// Handler for immediate notification
async fn notify_now(
    State(state): State<AppState>,
    Json(req): Json<NotifyRequest>,
) -> Result<Json<ApiResponse<NotifyResponse>>, StatusCode> {
    match state.notifier.send(&req).await {
        Ok(resp) => Ok(Json(ApiResponse::success(NotifyResponse {
            code: resp.code,
            message: resp.message,
        }))),
        Err(e) => {
            tracing::error!("Failed to send notification: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

// Handler for scheduling a cron job
async fn schedule_cron(
    State(state): State<AppState>,
    Json(req): Json<ScheduleRequest>,
) -> Result<Json<ApiResponse<JobCreatedResponse>>, StatusCode> {
    match state.scheduler.add_cron_job(req).await {
        Ok(job_id) => Ok(Json(ApiResponse::success(JobCreatedResponse { job_id }))),
        Err(e) => {
            tracing::error!("Failed to schedule cron job: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

// Handler for scheduling a one-time job
async fn schedule_one_time(
    State(state): State<AppState>,
    Json(req): Json<OneTimeRequest>,
) -> Result<Json<ApiResponse<JobCreatedResponse>>, StatusCode> {
    match state.scheduler.add_one_time_job(req).await {
        Ok(job_id) => Ok(Json(ApiResponse::success(JobCreatedResponse { job_id }))),
        Err(e) => {
            tracing::error!("Failed to schedule one-time job: {}", e);
            Ok(Json(ApiResponse::error(e.to_string())))
        }
    }
}

// Handler for listing all jobs
async fn list_jobs(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<crate::scheduler::ScheduledJob>>>, StatusCode> {
    let jobs = state.scheduler.list_jobs().await;
    Ok(Json(ApiResponse::success(jobs)))
}

// Handler for getting a specific job
async fn get_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<ApiResponse<crate::scheduler::ScheduledJob>>, StatusCode> {
    match state.scheduler.get_job(&job_id).await {
        Some(job) => Ok(Json(ApiResponse::success(job))),
        None => Ok(Json(ApiResponse::error("Job not found"))),
    }
}

// Handler for removing a job
async fn remove_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match state.scheduler.remove_job(&job_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

// Health check
async fn health() -> &'static str {
    "OK"
}

// Get device key info (without exposing the full key)
async fn device_info(State(state): State<AppState>) -> Json<serde_json::Value> {
    let key = &state.notifier.device_key;
    let masked = if key.len() > 8 {
        format!("{}...{}", &key[..4], &key[key.len()-4..])
    } else {
        "***".to_string()
    };
    
    Json(serde_json::json!({
        "device_key": masked,
        "status": "active"
    }))
}

pub fn create_router(state: AppState, auth_state: AuthState) -> Router {
    // 公开路由（不需要认证）
    let public_routes = Router::new()
        .route("/", get(|| async { "Agent Bark API" }))
        .route("/health", get(health));

    // 需要认证的路由
    let protected_routes = Router::new()
        .route("/device", get(device_info))
        // Immediate notification
        .route("/notify", post(notify_now))
        // Scheduled notifications
        .route("/schedule/cron", post(schedule_cron))
        .route("/schedule/once", post(schedule_one_time))
        // Job management
        .route("/jobs", get(list_jobs))
        .route("/jobs/:job_id", get(get_job).delete(remove_job))
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware));

    public_routes
        .merge(protected_routes)
        .with_state(state)
}
