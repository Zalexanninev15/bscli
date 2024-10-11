use std::fs::{ self };
use std::path::Path;
use std::process::Command;

use crate::main::DOWNLOAD_BASE_URL;
use crate::main::PLUGINS_DIR;
use crate::network::fetch_repo_data;
use crate::network::download_file;
use crate::parser::find_plugin;
use crate::tools::find_main_file;
use crate::tools::extract_zip;
use crate::plugins::read_first_line;
use crate::plugins::append_to_plugins_inst;
use crate::parser::fetch_apps_data;
use crate::parser::find_app;

pub async fn install_plugin(id_or_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo_data = fetch_repo_data().await?;
    let plugin = find_plugin(&repo_data, id_or_name)?;

    let file_name = plugin["file"].as_str().unwrap();
    let id = plugin["id"].as_str().unwrap();

    let download_url = format!("{}/{}/{}.zip", DOWNLOAD_BASE_URL, id, file_name);
    println!("Downloading from URL: {}", download_url);
    let temp_file = download_file(&download_url, &format!("{}.zip", file_name)).await?;

    println!("Download completed. Temp file path: {}", temp_file.display());
    if !temp_file.exists() {
        return Err(
            format!("Temp file does not exist after download: {}", temp_file.display()).into()
        );
    }

    let plugin_dir = format!("{}\\{}", PLUGINS_DIR, file_name);
    println!("Extracting zip to: {}", plugin_dir);
    extract_zip(&temp_file, &plugin_dir).map_err(|e| format!("Failed to extract zip: {}", e))?;

    let main_file = find_main_file(&plugin_dir).map_err(|e|
        format!("Failed to find main file: {}", e)
    )?;
    println!("Main file found: {}", main_file);
    let first_line = read_first_line(&main_file)?;
    append_to_plugins_inst(&first_line)?;

    println!("Plugin '{}' installed successfully!", id_or_name);
    Ok(())
}

pub async fn install_local_plugin(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = Path::new(file_path).file_stem().unwrap().to_str().unwrap();
    let plugin_dir = format!("{}\\{}", PLUGINS_DIR, file_name);

    println!("Extracting local zip to: {}", plugin_dir);
    extract_zip(Path::new(file_path), &plugin_dir).map_err(|e|
        format!("Failed to extract zip: {}", e)
    )?;

    let main_file = find_main_file(&plugin_dir).map_err(|e|
        format!("Failed to find main file: {}", e)
    )?;
    println!("Main file found: {}", main_file);
    let first_line = read_first_line(&main_file)?;
    append_to_plugins_inst(&first_line)?;

    println!("Local plugin '{}' installed successfully!", file_name);
    Ok(())
}

pub async fn install_software(software_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let apps_data = fetch_apps_data().await?;
    let app = find_app(&apps_data, software_name)?;

    let download_url = app["url"].as_str().ok_or("Invalid URL in app data")?;
    let name = app["name"].as_str().ok_or("Invalid name in app data")?;

    println!("Downloading {}...", name);
    let temp_file = download_file(download_url, &format!("{}.exe", name)).await?;

    println!("\n{} has been downloaded successfully.", name);
    println!("Running the installer...");
    let status = Command::new(temp_file.clone()).status().expect("Failed to execute installer");
    if status.success() {
        println!("Installation completed successfully.");
    } else {
        println!("Installation failed or was cancelled by the user.");
    }
    fs::remove_file(temp_file.as_os_str())?;

    Ok(())
}
