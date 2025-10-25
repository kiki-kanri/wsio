use tungstenite::protocol::WebSocketConfig;

pub mod atomic;
pub mod packet;
pub mod types;
pub mod utils;

pub fn channel_capacity_from_websocket_config(websocket_config: WebSocketConfig) -> usize {
    (websocket_config.max_write_buffer_size / websocket_config.write_buffer_size).clamp(64, 4096)
}
