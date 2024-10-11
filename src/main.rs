use std::env;
use std::path::Path;
use winconsole::console;

mod network;
mod installers;
mod parser;
mod plugins;
mod tools;

pub mod main {
    pub const REPO_URL: &str =
        "https://raw.githubusercontent.com/Zalexanninev15/Repository/refs/heads/main/repo.json";
    pub const APPS_URL: &str =
        "https://raw.githubusercontent.com/Zalexanninev15/Repository/refs/heads/main/apps.json";
    pub const DOWNLOAD_BASE_URL: &str =
        "https://github.com/Zalexanninev15/Repository/raw/refs/heads/main/SR5";
    pub const PLUGINS_DIR: &str = "D:\\Plugins";
    pub const PLUGINS_INST_FILE: &str = "D:\\Plugins\\plugins.inst";
    pub const CONFIG_URL: &str = "https://sharkremote.neocities.org/config";

    pub const UTILITY_NAME: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));
    pub const AUTHOR: &str = "Zalexanninev15 <blue.shark@disroot.org>";
    pub const LICENSE: &str = env!("CARGO_PKG_VERSION");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = console::set_title(main::UTILITY_NAME);

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "add" | "-a" | "--add" => {
            if args.len() < 3 {
                println!("Usage: {} add [id or name] [id or name] ...", main::UTILITY_NAME);
                return Ok(());
            }
            for id_or_name in &args[2..] {
                if id_or_name.starts_with("wlc://add") {
                    let id = id_or_name.trim_start_matches("wlc://add");
                    installers::install_plugin(id).await?;
                } else if Path::new(id_or_name).exists() {
                    installers::install_local_plugin(id_or_name).await?;
                } else {
                    installers::install_plugin(id_or_name).await?;
                }
            }
        }
        "install" | "-i" | "--install" => {
            if args.len() < 3 {
                println!("Usage: {} install [software_name]", main::UTILITY_NAME);
                return Ok(());
            }
            installers::install_software(&args[2]).await?;
        }
        "config" | "-c" | "--config" => {
            tools::open_config_page()?;
        }
        _ => {
            println!("Unknown command. Use '-h' or '--help' for usage information.");
        }
    }

    Ok(())
}

fn print_help() {
    println!("{} - Plugin and Software Manager", main::UTILITY_NAME);
    println!("Author: {}", main::AUTHOR);
    println!("License: {}", main::LICENSE);
    println!("\nUsage:");
    println!("  {} [command] [arguments]", main::UTILITY_NAME);
    println!("\nCommands:");
    println!("  add, -a, --add [id or name] [id or name] ...  Install plugins");
    println!("  install, -i, --install [software_name]        Install software");
    println!(
        "  config, -c, --config                          Open configuration page for Shark Remote 5"
    );
    println!("  -h, --help                                    Show this help message");
}
