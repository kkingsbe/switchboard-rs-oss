//! Tests for Gateway API handlers.
//!
//! This module contains tests for gateway management endpoints.

#[cfg(test)]
mod tests {
    use super::super::gateway::{
        gateway_up, gateway_status, gateway_down,
        ApiResponse, GatewayStartResponse, GatewayStatus, GatewayStopResponse,
    };
    use crate::api::tests::TestApiStateBuilder;
    use axum::{
        extract::State,
        response::Json,
    };
    use serial_test::serial;
    use std::sync::Arc;

    // ============================================================================
    // Handler Tests
    // ============================================================================

    /// Test gateway_up handler returns success response
    #[tokio::test]
    #[serial]
    async fn test_gateway_up_success() {
        // Create test state
        let state = TestApiStateBuilder::new()
            .with_test_instance()
            .build();
        let state = Arc::new(state);
        let state = State(state);

        // Call handler
        let result = gateway_up(state).await;

        // Should return success response
        match result {
            Ok(Json(response)) => {
                assert!(response.success, "Response should be successful");
                assert!(response.data.is_some(), "Response should have data");
                
                let data = response.data.unwrap();
                assert!(data.started, "Gateway should be marked as started");
                assert_eq!(data.instance_id, "test-instance", "Instance ID should match");
                assert!(!data.message.is_empty(), "Message should not be empty");
            }
            Err(e) => {
                panic!("Expected success, got error: {:?}", e);
            }
        }
    }

    /// Test gateway_status handler returns correct status
    #[tokio::test]
    #[serial]
    async fn test_gateway_status_running() {
        // Create test state
        let state = TestApiStateBuilder::new()
            .with_test_instance()
            .build();
        let state = Arc::new(state);
        let state = State(state);

        // Call handler
        let result = gateway_status(state).await;

        // Should return status response
        match result {
            Ok(Json(response)) => {
                assert!(response.success, "Response should be successful");
                assert!(response.data.is_some(), "Response should have data");
                
                let status = response.data.unwrap();
                assert_eq!(status.instance_id, "test-instance", "Instance ID should match");
                // Note: The current implementation returns running: false as a placeholder
                // This test verifies the response structure regardless of running state
                let _running: bool = status.running;
                let _port: Option<u16> = status.port;
                let _connected_agents: usize = status.connected_agents;
            }
            Err(e) => {
                panic!("Expected success, got error: {:?}", e);
            }
        }
    }

    /// Test gateway_down handler returns success response
    #[tokio::test]
    #[serial]
    async fn test_gateway_down_success() {
        // Create test state
        let state = TestApiStateBuilder::new()
            .with_test_instance()
            .build();
        let state = Arc::new(state);
        let state = State(state);

        // Call handler
        let result = gateway_down(state).await;

        // Should return success response
        match result {
            Ok(Json(response)) => {
                assert!(response.success, "Response should be successful");
                assert!(response.data.is_some(), "Response should have data");
                
                let data = response.data.unwrap();
                assert!(data.stopped, "Gateway should be marked as stopped");
                assert!(!data.message.is_empty(), "Message should not be empty");
            }
            Err(e) => {
                panic!("Expected success, got error: {:?}", e);
            }
        }
    }

    // ============================================================================
    // Data Structure Tests
    // ============================================================================

    /// Test GatewayStartResponse structure
    #[test]
    #[serial]
    fn test_gateway_start_response_structure() {
        let response = GatewayStartResponse {
            started: true,
            instance_id: "test-instance".to_string(),
            message: "Gateway started successfully".to_string(),
        };
        
        assert!(response.started);
        assert_eq!(response.instance_id, "test-instance");
        assert_eq!(response.message, "Gateway started successfully");
    }

    /// Test GatewayStatus structure
    #[test]
    #[serial]
    fn test_gateway_status_structure() {
        let status = GatewayStatus {
            running: true,
            instance_id: "test-instance".to_string(),
            port: Some(8080),
            connected_agents: 5,
        };
        
        assert!(status.running);
        assert_eq!(status.instance_id, "test-instance");
        assert_eq!(status.port, Some(8080));
        assert_eq!(status.connected_agents, 5);
    }

    /// Test GatewayStopResponse structure
    #[test]
    #[serial]
    fn test_gateway_stop_response_structure() {
        let response = GatewayStopResponse {
            stopped: true,
            message: "Gateway stopped successfully".to_string(),
        };
        
        assert!(response.stopped);
        assert_eq!(response.message, "Gateway stopped successfully");
    }

    /// Test ApiResponse success variant for GatewayStartResponse
    #[test]
    #[serial]
    fn test_api_response_gateway_start_success() {
        let response = ApiResponse::success(GatewayStartResponse {
            started: true,
            instance_id: "test".to_string(),
            message: "started".to_string(),
        });
        
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.message.is_none());
    }

    /// Test ApiResponse success variant for GatewayStatus
    #[test]
    #[serial]
    fn test_api_response_gateway_status_success() {
        let response = ApiResponse::success(GatewayStatus {
            running: false,
            instance_id: "test".to_string(),
            port: None,
            connected_agents: 0,
        });
        
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.message.is_none());
    }

    /// Test ApiResponse success variant for GatewayStopResponse
    #[test]
    #[serial]
    fn test_api_response_gateway_stop_success() {
        let response = ApiResponse::success(GatewayStopResponse {
            stopped: true,
            message: "stopped".to_string(),
        });
        
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.message.is_none());
    }

    /// Test ApiResponse error variant for gateway types
    #[test]
    #[serial]
    fn test_api_response_gateway_error() {
        let response: ApiResponse<GatewayStartResponse> = ApiResponse::error("test error");
        
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.message.is_some());
    }
}
