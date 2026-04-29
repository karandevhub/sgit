// This module handles the 'sgit uninstall' command. 
// It cleans up everything sgit ever created on your system.

use anyhow::{Context, Result};
use colored::Colorize;

pub async fn run() -> Result<()> {
    println!("\n  {} Starting uninstallation...", "→".cyan());

    // 1. Remove the data folder where the database and AI models are stored.
    let proj_dirs = directories::ProjectDirs::from("ai", "sgit", "sgit")
        .context("Could not determine application directory")?;
    
    let data_dir = proj_dirs.data_dir();
    if data_dir.exists() {
        print!("  {} Removing data directory ({})... ", "→".cyan(), data_dir.display());
        std::fs::remove_dir_all(data_dir).map_err(|e| {
            println!("{}", "Failed".red());
            e
        }).context("Failed to remove data directory")?;
        println!("{}", "Done".green());
    } else {
        println!("  {} Data directory not found, skipping.", "→".cyan());
    }

    // 2. Remove the Git hook if you happen to be in a repository that uses sgit.
    if let Ok(current_dir) = std::env::current_dir() {
        let hook_path = current_dir.join(".git").join("hooks").join("post-commit");
        if hook_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&hook_path) {
                // We only delete it if it's the one we installed.
                if content.contains("sgit index") {
                    print!("  {} Removing local post-commit hook... ", "→".cyan());
                    match std::fs::remove_file(&hook_path) {
                        Ok(_) => println!("{}", "Done".green()),
                        Err(e) => println!("{} ({})", "Failed".red(), e),
                    }
                }
            }
        }
    }

    // 3. Delete the sgit binary itself from your computer.
    print!("  {} Deleting sgit binary... ", "→".cyan());
    match self_replace::self_delete() {
        Ok(_) => println!("{}", "Done".green()),
        Err(e) => {
            println!("{} ({})", "Failed".red(), e);
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                println!(
                    "\n  {} Permission denied. Please run with sudo:\n     {} {} uninstall",
                    "✗".red(),
                    "sudo".yellow(),
                    "sgit".cyan()
                );
            }
            return Ok(()); // Stop here if binary deletion fails
        }
    }

    println!("\n  {} sgit has been successfully uninstalled.\n", "✓".green().bold());

    Ok(())
}
