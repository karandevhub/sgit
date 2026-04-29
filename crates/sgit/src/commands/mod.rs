pub mod index;
pub mod log;

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

pub mod hook {
    use anyhow::Result;
    use colored::Colorize;
    use std::fs;

    pub async fn run(path: Option<std::path::PathBuf>) -> Result<()> {
        let repo_path = path.unwrap_or_else(|| {
            std::env::current_dir().expect("Cannot read current directory")
        });
        let hook_path = repo_path.join(".git/hooks/post-commit");
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

pub mod update {
    use anyhow::Result;
    use colored::Colorize;

    pub async fn run() -> Result<()> {
        println!("\n  {} Checking for updates...\n", "→".cyan());

        let status = self_update::backends::github::Update::configure()
            .repo_owner("karandevhub")
            .repo_name("sgit")
            .bin_name("sgit")
            .show_download_progress(true)
            .current_version(env!("CARGO_PKG_VERSION"))
            .build()?
            .update()?;

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
