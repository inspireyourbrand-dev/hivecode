//! MCP JSON-RPC protocol implementation

use crate::error::{McpError, Result};
use crate::types::*;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

/// Build an initialize request for MCP protocol handshake
pub fn build_initialize_request(client_info: ClientInfo) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(1.into()),
        method: "initialize".to_string(),
        params: Some(json!(InitializeParams {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: json!({}),
            client_info,
        })),
    }
}

/// Build a tools list request
pub fn build_tools_list_request() -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(2.into()),
        method: "tools/list".to_string(),
        params: None,
    }
}

/// Build a tool call request
pub fn build_tool_call_request(name: String, args: Value) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(3.into()),
        method: "tools/call".to_string(),
        params: Some(json!(ToolCallParams { name, arguments: args })),
    }
}

/// Build a resources list request
pub fn build_resources_list_request() -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(4.into()),
        method: "resources/list".to_string(),
        params: None,
    }
}

/// Build a resource read request
pub fn build_resource_read_request(uri: String) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Value::Number(5.into()),
        method: "resources/read".to_string(),
        params: Some(json!(ResourceReadParams { uri })),
    }
}

/// Parse a JSON-RPC response and extract the typed result
pub fn parse_response<T: DeserializeOwned>(response: JsonRpcResponse) -> Result<T> {
    if let Some(error) = response.error {
        return Err(McpError::ServerError(format!(
            "JSON-RPC error {}: {}",
            error.code, error.message
        )));
    }

    response
        .result
        .ok_or_else(|| McpError::protocol("response has no result field"))?
        .try_into()
        .map_err(|e: serde_json::Error| McpError::SerializationError(e))
}

/// Deserialize response bytes into JsonRpcResponse
pub fn deserialize_response(bytes: &[u8]) -> Result<JsonRpcResponse> {
    serde_json::from_slice(bytes)
        .map_err(|e| McpError::protocol(format!("failed to parse response: {}", e)))
}

/// Serialize a request to JSON bytes
pub fn serialize_request(request: &JsonRpcRequest) -> Result<Vec<u8>> {
    serde_json::to_vec(request)
        .map_err(|e| McpError::protocol(format!("failed to serialize request: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_initialize_request() {
        let info = ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        };
        let req = build_initialize_request(info);
        assert_eq!(req.method, "initialize");
        assert_eq!(req.jsonrpc, "2.0");
    }

    #[test]
    fn test_build_tool_call_request() {
        let req = build_tool_call_request(
            "test_tool".to_string(),
            json!({"key": "value"}),
        );
        assert_eq!(req.method, "tools/call");
    }

    #[test]
    fn test_serialize_request() {
        let req = build_tools_list_request();
        let bytes = serialize_request(&req).unwrap();
        assert!(!bytes.is_empty());
        let deserialized: JsonRpcRequest = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(deserialized.method, "tools/list");
    }
}
