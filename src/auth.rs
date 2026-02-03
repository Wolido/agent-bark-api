use axum::{
    extract::{Query, Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

#[derive(Clone)]
pub struct AuthState {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthQuery {
    token: Option<String>,
}

pub async fn auth_middleware(
    State(state): State<AuthState>,
    Query(query): Query<AuthQuery>,
    request: Request,
    next: Next,
) -> Response {
    // 如果没有设置密码，直接放行
    if state.password.is_empty() {
        return next.run(request).await;
    }

    // 检查 query 参数 ?token=xxx
    if let Some(token) = query.token {
        if token == state.password {
            return next.run(request).await;
        }
    }

    // 检查 Authorization header: Bearer xxx
    if let Some(auth_header) = request.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            // 支持 "Bearer xxx" 或 "xxx"
            let token = auth_str
                .strip_prefix("Bearer ")
                .unwrap_or(auth_str)
                .trim();
            
            if token == state.password {
                return next.run(request).await;
            }
        }
    }

    // 验证失败
    (StatusCode::UNAUTHORIZED, "Unauthorized: invalid or missing token").into_response()
}
