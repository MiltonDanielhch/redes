//! Ubicación: `crates/infrastructure/src/middleware/rate_limit.rs`
//!
//! Descripción: Middleware de Rate Limiting con sliding window algorithm.
//!              Límites: 100 req/min global, 10 req/min para auth endpoints.
//!
//! ADRs relacionados: 0003 (Axum 0.8), 0009 (Rate Limiting)

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const WINDOW_SIZE_SECS: u64 = 60;
const GLOBAL_LIMIT: usize = 100;
const AUTH_LIMIT: usize = 10;

#[derive(Clone)]
pub struct RateLimiter {
    state: Arc<RwLock<RateLimiterState>>,
}

struct RateLimiterState {
    requests: HashMap<String, RateLimitEntry>,
    last_cleanup: Instant,
}

struct RateLimitEntry {
    timestamps: Vec<Instant>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(RateLimiterState {
                requests: HashMap::new(),
                last_cleanup: Instant::now(),
            })),
        }
    }

    pub async fn check_rate_limit(&self, key: &str, limit: usize) -> Result<RateLimitResult, RateLimitError> {
        let mut state = self.state.write().await;
        let now = Instant::now();

        if now.duration_since(state.last_cleanup) > Duration::from_secs(WINDOW_SIZE_SECS) {
            state.cleanup(now);
            state.last_cleanup = now;
        }

        let entry = state.requests.entry(key.to_string()).or_insert_with(|| RateLimitEntry {
            timestamps: Vec::new(),
        });

        let window_start = now - Duration::from_secs(WINDOW_SIZE_SECS);
        entry.timestamps.retain(|&t| t > window_start);

        let current_count = entry.timestamps.len();

        if current_count >= limit {
            let retry_after = entry
                .timestamps
                .first()
                .map(|t| {
                    let elapsed = now.duration_since(*t).as_secs();
                    WINDOW_SIZE_SECS.saturating_sub(elapsed)
                })
                .unwrap_or(0);

            return Err(RateLimitError::LimitExceeded {
                limit,
                remaining: 0,
                retry_after,
            });
        }

        entry.timestamps.push(now);

        Ok(RateLimitResult {
            allowed: true,
            limit,
            remaining: limit - entry.timestamps.len(),
            reset_in: WINDOW_SIZE_SECS,
        })
    }

    pub async fn check_global(&self, client_ip: &str) -> Result<RateLimitResult, RateLimitError> {
        self.check_rate_limit(&format!("global:{}", client_ip), GLOBAL_LIMIT).await
    }

    pub async fn check_auth(&self, client_ip: &str) -> Result<RateLimitResult, RateLimitError> {
        self.check_rate_limit(&format!("auth:{}", client_ip), AUTH_LIMIT).await
    }
}

impl RateLimiterState {
    fn cleanup(&mut self, now: Instant) {
        let window_start = now - Duration::from_secs(WINDOW_SIZE_SECS);
        self.requests.retain(|_, entry| {
            entry.timestamps.iter().any(|&t| t > window_start)
        });
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub limit: usize,
    pub remaining: usize,
    pub reset_in: u64,
}

#[derive(Debug)]
pub enum RateLimitError {
    LimitExceeded {
        limit: usize,
        remaining: usize,
        retry_after: u64,
    },
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        use axum::{
            http::StatusCode,
            Json,
        };

        let (status, body, retry_after) = match self {
            RateLimitError::LimitExceeded {
                limit: _,
                remaining: _,
                retry_after,
            } => (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": "Rate limit exceeded",
                    "retry_after": retry_after
                })),
                retry_after,
            ),
        };

        let mut response = (status, body).into_response();
        let headers = response.headers_mut();
        headers.insert(
            axum::http::header::HeaderName::from_static("x-ratelimit-remaining"),
            axum::http::header::HeaderValue::from_static("0"),
        );
        headers.insert(
            axum::http::header::HeaderName::from_static("retry-after"),
            axum::http::header::HeaderValue::from_str(&retry_after.to_string()).unwrap(),
        );

        response
    }
}

pub struct RateLimitMiddleware {
    limiter: RateLimiter,
    limit_type: RateLimitType,
}

#[derive(Clone, Copy)]
pub enum RateLimitType {
    Global,
    Auth,
}

impl RateLimitMiddleware {
    pub fn new(limit_type: RateLimitType) -> Self {
        Self {
            limiter: RateLimiter::new(),
            limit_type,
        }
    }

    pub async fn layer(self, request: Request, next: Next) -> Response {
        let client_ip = extract_client_ip(&request);

        let result = match self.limit_type {
            RateLimitType::Global => self.limiter.check_global(&client_ip).await,
            RateLimitType::Auth => self.limiter.check_auth(&client_ip).await,
        };

        match result {
            Ok(rate_limit) => {
                let mut response = next.run(request).await;
                let headers = response.headers_mut();
                headers.insert(
                    axum::http::header::HeaderName::from_static("x-ratelimit-limit"),
                    axum::http::header::HeaderValue::from_str(&rate_limit.limit.to_string()).unwrap(),
                );
                headers.insert(
                    axum::http::header::HeaderName::from_static("x-ratelimit-remaining"),
                    axum::http::header::HeaderValue::from_str(&rate_limit.remaining.to_string()).unwrap(),
                );
                response
            }
            Err(e) => e.into_response(),
        }
    }
}

fn extract_client_ip(request: &Request) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
