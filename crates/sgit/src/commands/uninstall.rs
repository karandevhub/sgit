
use anyhow::{Context, Result};
use colored::Colorize;

pub async fn run() -> Result<()> {
    println!("\n  {} Starting uninstallation...", "→".cyan());

    // Remove data directory.
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

    // Remove Git hook.
    if let Ok(current_dir) = std::env::current_dir() {
        let hook_path = current_dir.join(".git").join("hooks").join("post-commit");
        if hook_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&hook_path) {
                // Only delete if it's our hook.
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

    // Delete binary.
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
            return Ok(());
        }
    }

    println!("\n  {} sgit has been successfully uninstalled.\n", "✓".green().bold());

    Ok(())
}
