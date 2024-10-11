use reqwest;
use serde_json::Value;

use crate::main::APPS_URL;

pub async fn fetch_apps_data() -> Result<Value, Box<dyn std::error::Error>> {
    let resp = reqwest::get(APPS_URL).await?.text().await?;
    serde_json
        ::from_str(&resp)
        .map_err(|e| {
            format!(
                "Failed to parse apps data: {}. Please check the JSON structure at {}",
                e,
                APPS_URL
            ).into()
        })
}

pub fn find_app<'a>(
    apps_data: &'a Value,
    name: &str
) -> Result<&'a Value, Box<dyn std::error::Error>> {
    let apps = apps_data["apps"].as_array().ok_or("Invalid apps data structure")?;
    let name_lowercase = name.to_lowercase();
    apps.iter()
        .find(|app| {
            app["name"].as_str().map_or(false, |app_name| app_name.to_lowercase() == name_lowercase)
        })
        .ok_or_else(|| format!("Software '{}' not found in the apps list", name).into())
}

pub fn find_plugin<'a>(
    repo_data: &'a Value,
    id_or_name: &str
) -> Result<&'a Value, Box<dyn std::error::Error>> {
    let files = repo_data["files"].as_array().ok_or("Invalid repo data")?;
    files
        .iter()
        .find(|f| {
            let id_match = f["id"].as_str().map_or(false, |id| id == id_or_name);
            let name_match = f["file"].as_str().map_or(false, |name| name == id_or_name);
            id_match || name_match
        })
        .ok_or_else(|| "Plugin not found".into())
}