use anyhow::{Context, Result};
use colored::Colorize;

pub async fn run() -> Result<()> {
    println!("\n  {} Starting uninstallation...", "→".cyan());

    // 1. Remove the data directory
    let proj_dirs = directories::ProjectDirs::from("ai", "sgit", "sgit")
        .context("Could not determine application directory")?;
    
    let data_dir = proj_dirs.data_dir();
    if data_dir.exists() {
        print!("  {} Removing data directory ({})... ", "→".cyan(), data_dir.display());
        match std::fs::remove_dir_all(data_dir) {
            Ok(_) => println!("{}", "Done".green()),
            Err(e) => println!("{} ({})", "Failed".red(), e),
        }
    } else {
        println!("  {} Data directory not found, skipping.", "→".cyan());
    }

    // 2. Remove the post-commit hook if it exists in the current repo
    if let Ok(current_dir) = std::env::current_dir() {
        let hook_path = current_dir.join(".git").join("hooks").join("post-commit");
        if hook_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&hook_path) {
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

    // 3. Delete the binary itself
    print!("  {} Deleting sgit binary... ", "→".cyan());
    match self_replace::self_delete() {
        Ok(_) => println!("{}", "Done".green()),
        Err(e) => println!("{} ({})", "Failed".red(), e),
    }

    println!("\n  {} sgit has been successfully uninstalled.\n", "✓".green().bold());

    Ok(())
}
