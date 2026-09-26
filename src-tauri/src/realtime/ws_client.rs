use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, instrument, warn};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Commands sent from the app to the WebSocket client.
#[derive(Debug, Clone)]
pub enum ClientCommand {
    SendMessage(String),
    Disconnect,
}

/// Incoming message from the cloud server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    MemberLocation {
        member_id: String,
        name: String,
        lat: f64,
        lng: f64,
        accuracy: f64,
        timestamp: String,
    },
    GeofenceAlert {
        place_id: String,
        place_name: String,
        member_id: String,
        event: String, // "enter" | "exit"
        timestamp: String,
    },
    SosAlert {
        alert_id: String,
        user_id: String,
        user_name: String,
        lat: f64,
        lng: f64,
        silent: bool,
        timestamp: String,
    },
    ChatMessage {
        room_id: String,
        sender_id: String,
        body: String,
        media_urls: Option<Vec<String>>,
        timestamp: String,
    },
    IncidentAlert {
        incident_id: String,
        user_id: String,
        incident_type: String,
        lat: f64,
        lng: f64,
        severity: String,
        timestamp: String,
    },
    Pong,
}

/// WebSocket client that manages a persistent connection to the cloud server.
pub struct WsClient {
    url: String,
    token: String,
    connected: Arc<AtomicBool>,
}

impl WsClient {
    pub fn new(url: String, token: String, connected: Arc<AtomicBool>) -> Self {
        Self {
            url,
            token,
            connected,
        }
    }

    /// Connect to the WebSocket server with automatic reconnection.
    #[instrument(skip(self))]
    pub async fn run(&self, mut command_rx: mpsc::UnboundedReceiver<ClientCommand>) {
        let mut reconnect_delay = std::time::Duration::from_secs(1);

        loop {
            if !self.connected.load(Ordering::SeqCst) {
                info!("Realtime client stopped");
                break;
            }

            match self.connect_and_serve(&mut command_rx).await {
                Ok(()) => {
                    info!("WebSocket connection closed gracefully");
                    break;
                }
                Err(e) => {
                    error!(error = %e, "WebSocket connection error");
                }
            }

            // Exponential backoff reconnection
            warn!(
                delay_secs = reconnect_delay.as_secs(),
                "Reconnecting to WebSocket server"
            );
            tokio::time::sleep(reconnect_delay).await;
            reconnect_delay = std::cmp::min(reconnect_delay * 2, std::time::Duration::from_secs(60));
        }
    }

    async fn connect_and_serve(
        &self,
        command_rx: &mut mpsc::UnboundedReceiver<ClientCommand>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ws_url = format!("{}/ws?token={}", self.url, self.token);
        info!(url = %self.url, "Connecting to WebSocket server");

        let (mut ws_stream, _) = connect_async(&ws_url).await?;
        self.connected.store(true, Ordering::SeqCst);
        info!("WebSocket connected");

        // Send auth message
        let auth_msg = serde_json::json!({
            "type": "auth",
            "token": self.token,
        });
        ws_stream
            .send(Message::Text(auth_msg.to_string()))
            .await?;

        loop {
            tokio::select! {
                // Incoming messages from server
                Some(msg) = ws_stream.next() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            self.handle_server_message(&text).await;
                        }
                        Ok(Message::Close(_)) => {
                            info!("Server closed connection");
                            break;
                        }
                        Err(e) => {
                            error!(error = %e, "WebSocket receive error");
                            break;
                        }
                        _ => {}
                    }
                }

                // Commands from the app
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        ClientCommand::SendMessage(text) => {
                            if let Err(e) = ws_stream.send(Message::Text(text)).await {
                                error!(error = %e, "Failed to send message");
                            }
                        }
                        ClientCommand::Disconnect => {
                            info!("Disconnect requested");
                            let _ = ws_stream.close(None).await;
                            break;
                        }
                    }
                }
            }
        }

        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn handle_server_message(&self, text: &str) {
        match serde_json::from_str::<ServerMessage>(text) {
            Ok(msg) => {
                debug!(msg_type = ?std::mem::discriminant(&msg), "Received server message");
                super::handlers::handle_message(msg).await;
            }
            Err(e) => {
                warn!(error = %e, text = %text, "Failed to parse server message");
            }
        }
    }
}
