use std::fs::{ self, File };
use std::io::{ self };
use std::path::Path;
use zip::ZipArchive;
use std::process::Command;

use crate::main::CONFIG_URL;

pub fn open_config_page() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(CONFIG_URL).spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        println!("Opening configuration page in browser is only supported on Windows.");
        println!("Please open this URL manually: {}", CONFIG_URL);
    }

    Ok(())
}

pub fn extract_zip(zip_path: &Path, extract_to: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_path).map_err(|e|
        format!("Failed to open zip file '{}': {}", zip_path.display(), e)
    )?;
    let mut archive = ZipArchive::new(file)?;

    fs
        ::create_dir_all(extract_to)
        .map_err(|e| format!("Failed to create directory '{}': {}", extract_to, e))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = Path::new(extract_to).join(file.name());

        if file.name().ends_with('/') {
            fs
                ::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory '{}': {}", outpath.display(), e))?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs
                        ::create_dir_all(p)
                        .map_err(|e|
                            format!("Failed to create parent directory '{}': {}", p.display(), e)
                        )?;
                }
            }
            let mut outfile = File::create(&outpath).map_err(|e|
                format!("Failed to create file '{}': {}", outpath.display(), e)
            )?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

pub fn find_main_file(plugin_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
    for ext in &["ps1", "au3"] {
        let path = format!("{}\\main.{}", plugin_dir, ext);
        if Path::new(&path).exists() {
            return Ok(path);
        }
    }
    Err(format!("Main file not found in directory '{}'", plugin_dir).into())
}
