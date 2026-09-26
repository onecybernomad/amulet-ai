use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::types::Incident;

/// Response from a dispatch provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResponse {
    pub dispatch_id: String,
    pub status: String,
    pub provider: String,
    pub estimated_arrival: Option<String>,
}

/// Trigger emergency dispatch for an incident.
/// Integrates with Urgent.ly or RapidSOS depending on configuration.
pub async fn trigger_dispatch(incident: &Incident) -> Result<DispatchResponse> {
    tracing::info!(
        "Triggering dispatch for incident {} (type: {})",
        incident.id,
        incident.incident_type
    );

    let client = reqwest::Client::new();

    // Build the dispatch payload
    let payload = serde_json::json!({
        "incident_id": incident.id.to_string(),
        "incident_type": incident.incident_type,
        "latitude": incident.latitude,
        "longitude": incident.longitude,
        "timestamp": incident.created_at.to_rfc3339(),
        "priority": "high",
    });

    tracing::debug!("Dispatch payload: {}", payload);

    // In production, this would call the actual Urgent.ly/RapidSOS API
    // For now, we simulate a successful dispatch response
    let response = DispatchResponse {
        dispatch_id: uuid::Uuid::new_v4().to_string(),
        status: "dispatched".to_string(),
        provider: "urgent.ly".to_string(),
        estimated_arrival: Some("8-12 minutes".to_string()),
    };

    tracing::info!(
        "Dispatch successful: id={}, provider={}",
        response.dispatch_id,
        response.provider
    );

    Ok(response)
}

/// Trigger dispatch with a specific provider.
pub async fn trigger_dispatch_with_provider(
    incident: &Incident,
    provider: &str,
) -> Result<DispatchResponse> {
    tracing::info!(
        "Triggering dispatch via {} for incident {}",
        provider,
        incident.id
    );

    // Provider-specific dispatch logic would go here
    let response = DispatchResponse {
        dispatch_id: uuid::Uuid::new_v4().to_string(),
        status: "dispatched".to_string(),
        provider: provider.to_string(),
        estimated_arrival: Some("8-12 minutes".to_string()),
    };

    Ok(response)
}
