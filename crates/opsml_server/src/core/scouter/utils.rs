use crate::core::error::internal_server_error;
use crate::core::error::OpsmlServerError;
use crate::core::error::ServerError;
use anyhow::Result;
use axum::{http::StatusCode, Json};
use opsml_crypt::encrypt_file;
use opsml_storage::StorageClientEnum;
use opsml_types::DriftProfileUri;
use opsml_utils::FileUtils;
use scouter_client::{DriftProfile, DriftType};
use std::collections::HashMap;
use std::path::Path;
use tracing::debug;
use tracing::error;
use tracing::info;

pub fn find_drift_profile(
    files: &[String],
    drift_type: &str,
) -> Result<String, (StatusCode, Json<OpsmlServerError>)> {
    files
        .iter()
        .find(|f| f.as_str().contains(drift_type))
        .and_then(|f| std::path::Path::new(f.as_str()).file_name())
        .and_then(|f| f.to_str())
        .map(String::from)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(OpsmlServerError::no_drift_profile_found()),
            )
        })
}

pub async fn save_encrypted_profile(
    profile: &str,
    filename: &str,
    encryption_key: &[u8],
    storage_client: &StorageClientEnum,
    storage_path: &Path,
) -> Result<(), (StatusCode, Json<OpsmlServerError>)> {
    let tempdir = tempfile::tempdir().map_err(|e| {
        error!("Failed to create tempdir: {}", e);
        internal_server_error(e, "Failed to create tempdir")
    })?;

    let temp_path = tempdir.path().join(filename);

    // Write and encrypt file
    std::fs::write(&temp_path, profile).map_err(|e| {
        error!("Failed to write profile: {}", e);
        internal_server_error(e, "Failed to write profile")
    })?;

    encrypt_file(&temp_path, encryption_key).map_err(|e| {
        error!("Failed to encrypt file: {}", e);
        internal_server_error(e, "Failed to encrypt file")
    })?;

    // Save to storage
    let new_storage_path = storage_path.join(filename);
    storage_client
        .put(&temp_path, &new_storage_path, false)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(OpsmlServerError::failed_to_save_to_storage(e)),
            )
        })?;

    Ok(())
}

pub fn load_drift_profiles(
    path: &Path,
    drift_profile_uri_map: &HashMap<String, DriftProfileUri>,
) -> Result<HashMap<DriftType, DriftProfile>, ServerError> {
    let profiles = drift_profile_uri_map
        .values()
        .map(|uri| {
            let filepath = path.join(&uri.uri);
            let file = std::fs::read_to_string(&filepath)?;
            DriftProfile::from_str(uri.drift_type.clone(), file)
                .map_err(|e| {
                    error!("Failed to load drift profile: {}", e);
                    ServerError::LoadDriftProfileError(e.to_string())
                })
                .map(|profile| (uri.drift_type.clone(), profile))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    debug!("Loaded drift profiles: {:?}", profiles);
    Ok(profiles)
}
