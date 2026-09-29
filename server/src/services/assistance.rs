use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Types of roadside assistance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssistanceType {
    Towing,
    FlatTire,
    JumpStart,
    Lockout,
    FuelDelivery,
    MedicalAdvice,
}

impl std::fmt::Display for AssistanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssistanceType::Towing => write!(f, "towing"),
            AssistanceType::FlatTire => write!(f, "flat_tire"),
            AssistanceType::JumpStart => write!(f, "jump_start"),
            AssistanceType::Lockout => write!(f, "lockout"),
            AssistanceType::FuelDelivery => write!(f, "fuel_delivery"),
            AssistanceType::MedicalAdvice => write!(f, "medical_advice"),
        }
    }
}

/// Request for roadside or medical assistance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistanceRequest {
    pub user_id: String,
    pub assistance_type: AssistanceType,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub vehicle_info: Option<String>,
}

/// Response from an assistance provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistanceResponse {
    pub request_id: String,
    pub status: String, // "pending" | "accepted" | "en_route" | "arrived" | "completed" | "cancelled"
    pub provider: String,
    pub provider_phone: Option<String>,
    pub estimated_arrival: Option<String>,
    pub cost_estimate: Option<String>,
}

/// Status of an assistance request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistanceStatus {
    pub request_id: String,
    pub assistance_type: String,
    pub status: String,
    pub provider: String,
    pub provider_phone: Option<String>,
    pub estimated_arrival: Option<String>,
    pub cost_estimate: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Check if a subscription tier has roadside assistance.
/// Gold and Platinum tiers have roadside assistance.
pub fn tier_has_roadside(tier: &str) -> bool {
    matches!(tier, "gold" | "platinum")
}

/// Check if a subscription tier has medical advice.
/// Only Platinum tier has medical advice.
pub fn tier_has_medical(tier: &str) -> bool {
    matches!(tier, "platinum")
}

/// Request roadside assistance.
pub async fn request_roadside_assistance(
    req: &AssistanceRequest,
) -> Result<AssistanceResponse> {
    tracing::info!(
        "Roadside assistance requested: type={:?} at ({}, {})",
        req.assistance_type,
        req.latitude,
        req.longitude
    );

    // In production: integrate with roadside assistance provider API
    // For now, simulate a successful request
    let response = AssistanceResponse {
        request_id: uuid::Uuid::new_v4().to_string(),
        status: "accepted".to_string(),
        provider: "AAA Roadside".to_string(),
        provider_phone: Some("1-800-AAA-HELP".to_string()),
        estimated_arrival: Some("25-35 minutes".to_string()),
        cost_estimate: match req.assistance_type {
            AssistanceType::Towing => Some("$75-150".to_string()),
            AssistanceType::FlatTire => Some("$50-100".to_string()),
            AssistanceType::JumpStart => Some("$50-75".to_string()),
            AssistanceType::Lockout => Some("$75-125".to_string()),
            AssistanceType::FuelDelivery => Some("$25-50 + fuel cost".to_string()),
            AssistanceType::MedicalAdvice => Some("Included in Platinum".to_string()),
        },
    };

    tracing::info!(
        "Roadside assistance accepted: id={}, provider={}",
        response.request_id,
        response.provider
    );

    Ok(response)
}

/// Request medical advice.
pub async fn request_medical_advice(
    req: &AssistanceRequest,
) -> Result<AssistanceResponse> {
    tracing::info!(
        "Medical advice requested at ({}, {})",
        req.latitude,
        req.longitude
    );

    // In production: integrate with medical advice hotline API
    let response = AssistanceResponse {
        request_id: uuid::Uuid::new_v4().to_string(),
        status: "accepted".to_string(),
        provider: "Amulet Medical Hotline".to_string(),
        provider_phone: Some("1-800-AMULET-MD".to_string()),
        estimated_arrival: Some("Immediate — calling now".to_string()),
        cost_estimate: Some("Included in Platinum plan".to_string()),
    };

    tracing::info!(
        "Medical advice connected: id={}, provider={}",
        response.request_id,
        response.provider
    );

    Ok(response)
}

/// Get the status of an assistance request.
pub async fn get_assistance_status(request_id: &str) -> Result<AssistanceStatus> {
    tracing::info!("Getting assistance status for: {}", request_id);

    // In production: query the assistance provider API
    Ok(AssistanceStatus {
        request_id: request_id.to_string(),
        assistance_type: "towing".to_string(),
        status: "en_route".to_string(),
        provider: "AAA Roadside".to_string(),
        provider_phone: Some("1-800-AAA-HELP".to_string()),
        estimated_arrival: Some("15-20 minutes".to_string()),
        cost_estimate: Some("$75-150".to_string()),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Cancel an assistance request.
pub async fn cancel_assistance(request_id: &str) -> Result<AssistanceStatus> {
    tracing::info!("Cancelling assistance request: {}", request_id);

    // In production: call the assistance provider API to cancel
    Ok(AssistanceStatus {
        request_id: request_id.to_string(),
        assistance_type: "towing".to_string(),
        status: "cancelled".to_string(),
        provider: "AAA Roadside".to_string(),
        provider_phone: None,
        estimated_arrival: None,
        cost_estimate: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// Get available service providers near a location.
pub async fn get_nearby_providers(
    latitude: f64,
    longitude: f64,
    assistance_type: &AssistanceType,
) -> Result<Vec<ServiceProvider>> {
    tracing::info!(
        "Finding providers for {:?} near ({}, {})",
        assistance_type,
        latitude,
        longitude
    );

    // In production: query a provider database or API
    let providers = vec![
        ServiceProvider {
            id: "provider_1".to_string(),
            name: "AAA Roadside".to_string(),
            distance_km: 2.5,
            rating: 4.8,
            estimated_arrival: "25-35 min".to_string(),
            phone: "1-800-AAA-HELP".to_string(),
        },
        ServiceProvider {
            id: "provider_2".to_string(),
            name: "Quick Tow Services".to_string(),
            distance_km: 4.2,
            rating: 4.5,
            estimated_arrival: "30-40 min".to_string(),
            phone: "1-800-QUICK-TOW".to_string(),
        },
    ];

    Ok(providers)
}

/// Service provider information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceProvider {
    pub id: String,
    pub name: String,
    pub distance_km: f64,
    pub rating: f64,
    pub estimated_arrival: String,
    pub phone: String,
}
