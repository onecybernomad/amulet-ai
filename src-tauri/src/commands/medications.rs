use crate::db::local_cache::LocalCache;
use crate::models::{DoseLog, Medication};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info, instrument};
use uuid::Uuid;

// ── Types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AddMedicationRequest {
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub time_of_day: Vec<String>,
    pub notes: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMedicationRequest {
    pub id: String,
    pub name: Option<String>,
    pub dosage: Option<String>,
    pub frequency: Option<String>,
    pub time_of_day: Option<Vec<String>>,
    pub notes: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct LogDoseRequest {
    pub med_id: String,
    pub status: String, // "taken" | "skipped" | "missed"
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

// ── Commands ──────────────────────────────────────────────────────────────

/// Get all medications for the current user.
#[tauri::command]
pub async fn get_medications(
    cache: State<'_, LocalCache>,
) -> Result<Vec<Medication>, String> {
    info!("Getting medications");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;
    let meds = cache.get_medications(&user.id).map_err(|e| e.to_string())?;
    debug!(count = meds.len(), "Retrieved medications");
    Ok(meds)
}

/// Add a new medication.
#[tauri::command]
pub async fn add_medication(
    req: AddMedicationRequest,
    cache: State<'_, LocalCache>,
) -> Result<Medication, String> {
    info!(name = %req.name, "Adding medication");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    let med = Medication {
        id: Uuid::new_v4().to_string(),
        user_id: user.id,
        name: req.name,
        dosage: req.dosage,
        frequency: req.frequency,
        time_of_day: req.time_of_day,
        notes: req.notes,
        color: req.color,
        icon: req.icon,
        active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    cache.insert_medication(&med).map_err(|e| e.to_string())?;
    Ok(med)
}

/// Update an existing medication.
#[tauri::command]
pub async fn update_medication(
    req: UpdateMedicationRequest,
    cache: State<'_, LocalCache>,
) -> Result<Medication, String> {
    info!(med_id = %req.id, "Updating medication");
    let mut med = cache
        .get_medication(&req.id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Medication not found".to_string())?;

    if let Some(name) = req.name {
        med.name = name;
    }
    if let Some(dosage) = req.dosage {
        med.dosage = dosage;
    }
    if let Some(frequency) = req.frequency {
        med.frequency = frequency;
    }
    if let Some(time_of_day) = req.time_of_day {
        med.time_of_day = time_of_day;
    }
    if let Some(notes) = req.notes {
        med.notes = Some(notes);
    }
    if let Some(color) = req.color {
        med.color = Some(color);
    }
    if let Some(icon) = req.icon {
        med.icon = Some(icon);
    }
    if let Some(active) = req.active {
        med.active = active;
    }
    med.updated_at = chrono::Utc::now();

    cache.update_medication(&med).map_err(|e| e.to_string())?;
    Ok(med)
}

/// Delete a medication.
#[tauri::command]
pub async fn delete_medication(
    med_id: String,
    cache: State<'_, LocalCache>,
) -> Result<(), String> {
    info!(med_id = %med_id, "Deleting medication");
    cache.delete_medication(&med_id).map_err(|e| e.to_string())?;
    Ok(())
}

/// Log a dose for a medication.
#[tauri::command]
pub async fn log_dose(
    req: LogDoseRequest,
    cache: State<'_, LocalCache>,
) -> Result<DoseLog, String> {
    info!(med_id = %req.med_id, status = %req.status, "Logging dose");
    let user = cache
        .get_current_user()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No authenticated user".to_string())?;

    let log = DoseLog {
        id: Uuid::new_v4().to_string(),
        medication_id: req.med_id,
        user_id: user.id,
        status: req.status,
        timestamp: req.timestamp.unwrap_or_else(chrono::Utc::now),
    };

    cache.insert_dose_log(&log).map_err(|e| e.to_string())?;
    Ok(log)
}
