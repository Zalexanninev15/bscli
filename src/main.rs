use std::env;
use std::fs::{ self, File };
use std::io::{ self, Write, BufRead, BufReader };
use std::path::{ Path, PathBuf };
use reqwest;
use serde_json::Value;
use indicatif::{ ProgressBar, ProgressStyle };
use zip::ZipArchive;
use futures_util::StreamExt;

const REPO_URL: &str =
    "https://raw.githubusercontent.com/Zalexanninev15/Repository/refs/heads/main/repo.json";
const DOWNLOAD_BASE_URL: &str =
    "https://github.com/Zalexanninev15/Repository/raw/refs/heads/main/SR5";
const PLUGINS_DIR: &str = "D:\\Plugins";
const PLUGINS_INST_FILE: &str = "D:\\Plugins\\plugins.inst";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args[1] != "install" {
        println!("Usage: install [id or name]");
        return Ok(());
    }

    let id_or_name = &args[2];
    let repo_data = fetch_repo_data().await?;
    let plugin = find_plugin(&repo_data, id_or_name)?;

    let file_name = plugin["file"].as_str().unwrap();
    let id = plugin["id"].as_str().unwrap();

    let download_url = format!("{}/{}/{}.zip", DOWNLOAD_BASE_URL, id, file_name);
    println!("Downloading from URL: {}", download_url);
    let temp_file = download_file(&download_url).await?;

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

    println!("Plugin installed successfully!");
    Ok(())
}

async fn fetch_repo_data() -> Result<Value, Box<dyn std::error::Error>> {
    let resp = reqwest::get(REPO_URL).await?.text().await?;
    Ok(serde_json::from_str(&resp)?)
}

fn find_plugin<'a>(
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

async fn download_file(url: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?;
    let total_size = resp.content_length().unwrap_or(0);

    // Progress bar setup
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
            )?
            .progress_chars("#>-")
    );

    let temp_dir = std::env::temp_dir();
    let temp_file = tempfile::NamedTempFile::new_in(&temp_dir)?;
    let temp_path = temp_file.path().to_path_buf();
    let mut file = fs::File::create(&temp_path)?;

    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message("Download completed");

    // Persist the file to avoid deletion when temp_file goes out of scope
    let saved_path = temp_path.clone();
    temp_file.persist(&saved_path)?;

    println!("Downloaded file path: {:?}", saved_path);
    Ok(saved_path)
}

fn extract_zip(zip_path: &Path, extract_to: &str) -> Result<(), Box<dyn std::error::Error>> {
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

fn find_main_file(plugin_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
    for ext in &["ps1", "au3"] {
        let path = format!("{}\\main.{}", plugin_dir, ext);
        if Path::new(&path).exists() {
            return Ok(path);
        }
    }
    Err(format!("Main file not found in directory '{}'", plugin_dir).into())
}

fn read_first_line(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;
    Ok(first_line.trim().to_string())
}

fn append_to_plugins_inst(line: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::OpenOptions::new().create(true).append(true).open(PLUGINS_INST_FILE)?;
    writeln!(file, "{}", line)?;
    Ok(())
}
