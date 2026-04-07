use anyhow::{Context, Result, anyhow, bail};
use rs_firebase_admin_sdk::{App, auth::FirebaseAuth, client::ReqwestApiClient};

use crate::config::ResolvedConnection;

pub enum AuthBackend {
    Emulator {
        host: String,
    },
    Live {
        credentials_path: Option<String>,
        project_id: Option<String>,
    },
}

impl AuthBackend {
    pub fn from_resolved(conn: &ResolvedConnection) -> Self {
        if let Some(ref host) = conn.emulator_host {
            AuthBackend::Emulator { host: host.clone() }
        } else {
            AuthBackend::Live {
                credentials_path: conn.credentials.clone(),
                project_id: conn.project.clone(),
            }
        }
    }
}

pub async fn init_firebase(backend: AuthBackend) -> Result<FirebaseAuth<ReqwestApiClient>> {
    match backend {
        AuthBackend::Emulator { host } => {
            let url = if host.starts_with("http") {
                host
            } else {
                format!("http://{host}")
            };
            let app = App::emulated();
            Ok(app.auth(url))
        }
        AuthBackend::Live {
            credentials_path: Some(path),
            project_id,
        } => {
            let expanded = shellexpand::tilde(&path).to_string();
            let bytes = tokio::fs::read(&expanded)
                .await
                .context(format!("Failed to read credentials file: {expanded}"))?;
            let sa_json: serde_json::Value =
                serde_json::from_slice(&bytes).context("Invalid JSON in credentials file")?;

            let proj_id = project_id
                .or_else(|| sa_json["project_id"].as_str().map(String::from))
                .ok_or_else(|| anyhow!("project_id required when not in service account JSON"))?;

            use google_cloud_auth::credentials::service_account::{self, AccessSpecifier};
            use rs_firebase_admin_sdk::Credentials;

            let scopes = [
                "https://www.googleapis.com/auth/cloud-platform",
                "https://www.googleapis.com/auth/userinfo.email",
            ];

            let creds: Credentials = service_account::Builder::new(sa_json)
                .with_access_specifier(AccessSpecifier::from_scopes(scopes))
                .build_access_token_credentials()
                .context("Failed to build credentials from service account")?
                .into();

            let client = ReqwestApiClient::new(reqwest::Client::new(), creds);
            Ok(FirebaseAuth::live(&proj_id, client))
        }
        AuthBackend::Live {
            credentials_path: None,
            project_id: Some(id),
        } => {
            let app = App::live_with_project_id(&id)
                .await
                .map_err(|e| anyhow!("{e}"))
                .context("Failed to create Firebase app with project ID")?;
            Ok(app.auth())
        }
        AuthBackend::Live {
            credentials_path: None,
            project_id: None,
        } => {
            let app = App::live()
                .await
                .map_err(|e| anyhow!("{e}"))
                .context("Failed to create Firebase app with ADC")?;
            Ok(app.auth())
        }
    }
}

pub fn is_emulator(conn: &ResolvedConnection) -> bool {
    conn.emulator_host.is_some()
}

pub fn require_emulator(conn: &ResolvedConnection) -> Result<()> {
    if !is_emulator(conn) {
        bail!("This command is only available when connected to an emulator");
    }
    Ok(())
}
