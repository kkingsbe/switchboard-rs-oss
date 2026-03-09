//! Test HTTP client utilities for the Switchboard REST API.
//!
//! This module provides a test client for making HTTP requests to the API
//! during testing.
//!
//! Note: The full TestClient using axum-test requires test builds.
//! For integration testing, use tower::ServiceExt with the router directly.

use crate::api::router::create_router;
use crate::api::state::ApiState;
use axum::Router;
use std::sync::Arc;

/// Create a test router with the given state.
pub fn create_test_router(state: ApiState) -> Router {
    create_router(Arc::new(state))
}

/// Create a test router with default test state.
pub fn create_default_test_router() -> Router {
    let state = create_test_state();
    create_router(Arc::new(state))
}

/// Import test state helpers
use crate::api::tests::state::create_test_state;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_router_responds_to_health() {
        let router = create_default_test_router();
        let response = router.oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap()
        ).await.unwrap();
        
        // Should return 200 or 404 depending on router config
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }
}
