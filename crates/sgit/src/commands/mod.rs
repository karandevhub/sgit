pub mod index;
pub mod log;
pub mod uninstall;

// This module handles the 'sgit status' command.
pub mod status {
    use anyhow::Result;
    use colored::Colorize;
    use sgit_core::db::Store;

    pub async fn run() -> Result<()> {
        match Store::open() {
            Ok(store) => {
                let count = store.count().unwrap_or(0);
                println!("\n  {} {}", "Indexed commits:".dimmed(), count.to_string().bold());
                println!(
                    "  {} {}\n",
                    "DB:".dimmed(),
                    sgit_core::config::display_path(store.db_path()).dimmed()
                );
            }
            Err(e) => {
                println!("\n  {} {}\n", "No index found.".yellow(), e);
            }
        }
        Ok(())
    }
}

// This module handles the 'sgit hook' command.
// It installs a small script in .git/hooks/post-commit that triggers 
// an index update every time you make a commit.
pub mod hook {
    use anyhow::Result;
    use colored::Colorize;
    use std::fs;

    pub async fn run(path: Option<std::path::PathBuf>) -> Result<()> {
        let repo_path = path.unwrap_or_else(|| {
            std::env::current_dir().expect("Cannot read current directory")
        });
        let hook_path = repo_path.join(".git/hooks/post-commit");
        
        // The script just runs 'sgit index' in the background.
        let hook_content = "#!/bin/sh\nsgit index 2>/dev/null &\n";

        if hook_path.exists() {
            println!(
                "\n  {} post-commit hook already exists at {}\n",
                "!".yellow(),
                hook_path.display().to_string().dimmed()
            );
            return Ok(());
        }

        fs::write(&hook_path, hook_content)?;
        
        // On Linux/Mac, we need to make the script "executable".
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755))?;
        }

        println!(
            "\n  {} Hook installed at {}\n  {} on every commit.\n",
            "✓".green().bold(),
            hook_path.display().to_string().dimmed(),
            "sgit index will run automatically".dimmed()
        );
        Ok(())
    }
}

// This module handles the 'sgit install' command.
// It copies the running binary into a standard folder (like ~/.local/bin)
// so you can run 'sgit' from anywhere in your terminal.
pub mod install {
    use anyhow::Result;
    use colored::Colorize;
    use std::env;

    pub async fn run() -> Result<()> {
        let current = env::current_exe()?;
        let target = dirs_install_path();

        std::fs::copy(&current, &target)?;
        println!(
            "\n  {} Installed to {}\n",
            "✓".green().bold(),
            target.display().to_string().cyan()
        );
        Ok(())
    }

    /// Determines the best place to install the binary on your system.
    fn dirs_install_path() -> std::path::PathBuf {
        // Try ~/.local/bin first (Linux/Mac user-level), fall back to /usr/local/bin
        if let Some(home) = std::env::var_os("HOME") {
            let p = std::path::PathBuf::from(home).join(".local/bin/sgit");
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            return p;
        }
        std::path::PathBuf::from("/usr/local/bin/sgit")
    }
}

// This module handles the 'sgit update' command.
// It checks GitHub to see if there's a newer version of sgit available.
pub mod update {
    use anyhow::Result;
    use colored::Colorize;

    pub async fn run() -> Result<()> {
        println!("\n  {} Checking for updates...\n", "→".cyan());

        let target = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            "darwin-aarch64"
        } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            "darwin-x86_64"
        } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
            "windows-x86_64"
        } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            "linux-x86_64"
        } else {
            self_update::get_target()
        };

        let status = tokio::task::spawn_blocking(move || {
            self_update::backends::github::Update::configure()
                .repo_owner("karandevhub")
                .repo_name("sgit")
                .bin_name("sgit")
                .target(target)
                .show_download_progress(true)
                .current_version(env!("CARGO_PKG_VERSION"))
                .build()?
                .update()
        })
        .await??;

        match status {
            self_update::Status::UpToDate(v) => {
                println!("  {} Already up to date (v{}).\n", "✓".green().bold(), v);
            }
            self_update::Status::Updated(v) => {
                println!("  {} Updated to v{}!\n", "✓".green().bold(), v);
            }
        }
        Ok(())
    }
}
