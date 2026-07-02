//! ORBIT Desktop Agent (host).
//!
//! Captures the screen, encodes it, and streams it to a remote client over a
//! WebRTC data channel while applying the client's mouse/keyboard input. The
//! backend acts as the signaling relay; the client is the WebRTC offerer and
//! this agent answers.

mod capture;
mod config;
mod encoder;
mod input;
mod network;

use std::sync::Arc;
use std::time::Duration;

use config::AgentConfig;
use network::{SignalingClient, SignalingMode, WebRtcPeer};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SessionResponse {
    session_id: String,
}

#[derive(Serialize)]
struct RegisterDeviceRequest {
    device_name: String,
    os_version: String,
    public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_version: Option<String>,
}

#[derive(Serialize)]
struct CreateDeviceSessionRequest {
    host_device_id: String,
    client_device_id: String,
    session_type: String,
}

#[derive(Deserialize)]
struct RegisterDeviceResponse {
    device_id: String,
    device_token: String,
}

async fn register_device(config: &AgentConfig, user_token: &str) -> anyhow::Result<String> {
    let url = format!("http://{}/api/v1/devices/register", config.backend_host);
    let client = reqwest::Client::new();
    
    let device_name = std::env::var("ORBIT_DEVICE_NAME").unwrap_or_else(|_| {
        format!("{}-{}", std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown".to_string()), "Agent")
    });
    
    let os_version = get_os_version();
    let public_key = generate_dummy_key(); // In production, this should be a real key
    
    let request = RegisterDeviceRequest {
        device_name,
        os_version,
        public_key,
        device_type: Some("windows".to_string()),
        agent_version: Some("1.0.0".to_string()),
    };
    
    let res = client.post(&url)
        .header("Authorization", format!("Bearer {}", user_token))
        .json(&request)
        .send()
        .await?;
    
    if !res.status().is_success() {
        return Err(anyhow::anyhow!("Device registration failed: {}", res.status()));
    }
    
    let response: RegisterDeviceResponse = res.json().await?;
    tracing::info!(device_id = %response.device_id, "Device registered successfully");
    Ok(response.device_token)
}

async fn send_heartbeat(config: &AgentConfig, device_token: &str) {
    let url = format!("http://{}/api/v1/devices/heartbeat", config.backend_host);
    let client = reqwest::Client::new();
    
    loop {
        if let Err(e) = client.post(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .send()
            .await
        {
            tracing::debug!("heartbeat failed: {e}");
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

async fn fetch_session_id(config: &AgentConfig, device_token: &str) -> anyhow::Result<String> {
    let url = format!("http://{}/api/v1/sessions/active", config.backend_host);
    let client = reqwest::Client::new();

    loop {
        tracing::info!("polling for active session...");
        if let Ok(res) = client.get(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .send()
            .await {

            if res.status().is_success() {
                if let Ok(json) = res.json::<SessionResponse>().await {
                    return Ok(json.session_id);
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn create_device_to_device_session(
    config: &AgentConfig,
    user_token: &str,
    host_device_id: &str,
    client_device_id: &str,
) -> anyhow::Result<String> {
    let url = format!("http://{}/api/v1/sessions/create", config.backend_host);
    let client = reqwest::Client::new();

    let request = CreateDeviceSessionRequest {
        host_device_id: host_device_id.to_string(),
        client_device_id: client_device_id.to_string(),
        session_type: "device".to_string(),
    };

    let res = client.post(&url)
        .header("Authorization", format!("Bearer {}", user_token))
        .json(&request)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(anyhow::anyhow!("Failed to create device-to-device session: {}", res.status()));
    }

    let response: SessionResponse = res.json().await?;
    tracing::info!(session_id = %response.session_id, "Device-to-device session created");
    Ok(response.session_id)
}

async fn poll_pending_sessions(config: &AgentConfig, device_token: &str) -> anyhow::Result<Option<String>> {
    let url = format!("http://{}/api/v1/sessions/pending", config.backend_host);
    let client = reqwest::Client::new();

    let res = client.get(&url)
        .header("Authorization", format!("Bearer {}", device_token))
        .send()
        .await?;

    if res.status().is_success() {
        if let Ok(sessions) = res.json::<Vec<serde_json::Value>>().await {
            if !sessions.is_empty() {
                // Return the first pending session
                if let Some(session_id) = sessions[0].get("session_id").and_then(|s| s.as_str()) {
                    return Ok(Some(session_id.to_string()));
                }
            }
        }
    }

    Ok(None)
}

fn get_os_version() -> String {
    #[cfg(windows)]
    {
        use std::process::Command;
        let output = Command::new("cmd")
            .args(&["/C", "ver"])
            .output();
        match output {
            Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
            Err(_) => "Windows".to_string(),
        }
    }
    #[cfg(not(windows))]
    {
        "Unknown".to_string()
    }
}

fn generate_dummy_key() -> String {
    // In production, this should generate a real cryptographic key pair
    use uuid::Uuid;
    format!("orbit-key-{}", Uuid::new_v4())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = AgentConfig::from_env();
    tracing::info!(backend = %config.backend_host, "starting ORBIT desktop agent");

    // Check if we need to register the device
    let device_token = if let Ok(token) = std::env::var("ORBIT_DEVICE_TOKEN") {
        token
    } else if let Ok(user_token) = std::env::var("ORBIT_USER_TOKEN") {
        tracing::info!("No device token found, registering device...");
        register_device(&config, &user_token).await?
    } else {
        return Err(anyhow::anyhow!("ORBIT_DEVICE_TOKEN or ORBIT_USER_TOKEN must be set"));
    };

    // Start heartbeat in background
    let heartbeat_config = config.clone();
    let heartbeat_token = device_token.clone();
    tokio::spawn(async move {
        send_heartbeat(&heartbeat_config, &heartbeat_token).await;
    });

    // Determine mode: host (default) or client
    let mode = std::env::var("ORBIT_MODE").unwrap_or_else(|_| "host".to_string());
    let session_id = match mode.as_str() {
        "client" => {
            // Client mode: connect to another device
            let host_device_id = std::env::var("ORBIT_HOST_DEVICE_ID")
                .map_err(|_| anyhow::anyhow!("ORBIT_HOST_DEVICE_ID required for client mode"))?;
            let user_token = std::env::var("ORBIT_USER_TOKEN")
                .map_err(|_| anyhow::anyhow!("ORBIT_USER_TOKEN required for client mode"))?;
            let client_device_id = std::env::var("ORBIT_CLIENT_DEVICE_ID")
                .map_err(|_| anyhow::anyhow!("ORBIT_CLIENT_DEVICE_ID required for client mode"))?;

            tracing::info!("Running in client mode, connecting to device: {}", host_device_id);
            create_device_to_device_session(&config, &user_token, &host_device_id, &client_device_id).await?
        }
        "host" => {
            // Host mode: wait for incoming connections (default)
            // Check for pending sessions first (for device-to-device)
            if let Ok(Some(pending_session_id)) = poll_pending_sessions(&config, &device_token).await {
                tracing::info!(session_id = %pending_session_id, "found pending device-to-device session");
                pending_session_id
            } else if config.session_id.is_empty() {
                fetch_session_id(&config, &device_token).await?
            } else {
                config.session_id.clone()
            }
        }
        _ => {
            return Err(anyhow::anyhow!("Invalid ORBIT_MODE: must be 'host' or 'client'"));
        }
    };

    tracing::info!(session_id = %session_id, mode = %mode, "starting WebRTC connection");

    // Determine signaling mode based on agent mode
    let signaling_mode = match mode.as_str() {
        "client" => SignalingMode::Client,
        "host" => SignalingMode::Host,
        _ => SignalingMode::Host, // Default to host mode
    };

    // Build the peer connection with the configured ICE servers.
    let peer = Arc::new(
        WebRtcPeer::new(config.ice_servers.clone())
            .await
            .map_err(|e| anyhow::anyhow!("failed to create WebRTC peer: {e}"))?,
    );

    // Connect to the backend signaling relay and run until the session ends.
    let client = SignalingClient::new(&config.backend_host, peer, config.clone(), signaling_mode);
    client
        .connect(&session_id)
        .await
        .map_err(|e| anyhow::anyhow!("signaling failed: {e}"))?;

    Ok(())
}
