# sgit — Semantic Git History Search

[![GitHub downloads](https://img.shields.io/github/downloads/karandevhub/sgit/total?style=for-the-badge&color=cyan)](https://github.com/karandevhub/sgit/releases)
[![GitHub release](https://img.shields.io/github/v/release/karandevhub/sgit?style=for-the-badge&color=green)](https://github.com/karandevhub/sgit/releases)
[![License](https://img.shields.io/github/license/karandevhub/sgit?style=for-the-badge&color=blue)](LICENSE)

> **Find commits with natural language instead of grep patterns.**

<p align="center">
  <video src="https://github.com/karandevhub/sgit/raw/main/docs/assets/sgit-demo.mp4" width="100%" controls></video>
</p>

```bash
# Search by meaning, not just keywords
sgit log "where did we fix the login timeout issue?"
sgit log "show me changes related to database performance" -n 5
sgit log "how are we handling user permissions and security"
sgit log "recent work on the payment gateway" --author alice
```

---

## Why sgit?

`git log --grep` only matches exact strings. `sgit` understands *meaning*.

| | `git log --grep` | `sgit log` |
|---|---|---|
| Finds `"fix auth timeout"` with query `"login session expired"` | ✗ | ✓ |
| Needs a running service or API key | — | ✗ (100% offline) |
| Works on any git repo | ✓ | ✓ |
| Speed on 10 000 commits | instant | ~20 ms |

It runs a local **[ALL-MiniLM-L6-v2](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)** embedding model and stores all embeddings in a single SQLite file. No cloud, no API keys, no daemons.

---

## Installation

### macOS & Linux

```bash
curl -fsSL https://raw.githubusercontent.com/karandevhub/sgit/main/install.sh | bash
```

Install a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/karandevhub/sgit/main/install.sh | bash -s -- 0.1.14
```

### Windows (PowerShell)

```powershell
iwr -useb https://raw.githubusercontent.com/karandevhub/sgit/main/install.ps1 | iex
```

### From source (requires Rust stable)

```bash
git clone https://github.com/karandevhub/sgit
cd sgit
cargo install --path crates/sgit
```



## Quick start

```bash
# 1. Go into any git repository
cd ~/your-project

# 2. Build the search index (downloads ~80 MB model on first run)
sgit index

# 3. Search!
sgit log "when did the login system change?"
sgit log "I need to find where we improved the speed of the API"
sgit log "recent work on billing and money" --author bob -n 5
```

The index is incremental — re-run `sgit index` after adding new commits or install the auto-hook:

```bash
sgit hook   # installs a post-commit hook so the index updates automatically
```

---

## Commands

| Command | Description |
|---|---|
| `sgit index` | Build or update the semantic search index |
| `sgit index --full` | Full rebuild from scratch |
| `sgit log <query>` | Semantic search (alias: `sgit search`) |
| `sgit log <query> -n 20` | Return top 20 results |
| `sgit log <query> --author alice` | Filter by author name |
| `sgit log <query> --after 2024-01-01` | Filter by date |
| `sgit log <query> --show-scores` | Show similarity scores |
| `sgit status` | Show index stats |
| `sgit hook` | Install post-commit auto-index hook |
| `sgit update` | Update sgit to the latest release |
| `sgit uninstall` | Safely remove the app, index, and git hook |

---

## Building from source

```bash
# Prerequisites: Rust stable toolchain
rustup update stable

git clone https://github.com/karandevhub/sgit
cd sgit

# Development build
cargo build -p sgit

# Optimised release binary
cargo build -p sgit --release
# Binary at: target/release/sgit
```

Run tests:

```bash
cargo test
```

Run lints:

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

---

## Workspace layout

```
sgit/
├── Cargo.toml              # workspace root — all shared deps declared here
├── rust-toolchain.toml     # pins Rust stable
├── install.sh              # curl | bash installer (macOS / Linux)
├── install.ps1             # PowerShell installer (Windows)
├── crates/
│   ├── sgit/               # CLI binary (clap, colored, indicatif)
│   │   └── src/
│   │       ├── main.rs
│   │       ├── cli.rs      # clap command definitions
│   │       └── commands/   # index, log, status, hook, update, install
│   └── sgit-core/          # library crate (no CLI deps)
│       └── src/
│           ├── lib.rs
│           ├── config.rs   # cross-platform data paths
│           ├── error.rs    # typed SgitError enum
│           ├── db/         # SQLite Store (rusqlite)
│           ├── indexer/    # git reader + fastembed wrapper
│           └── search/     # cosine similarity engine (rayon)
└── .github/
    └── workflows/          # CI (cargo test + cross-compile releases)
```

---

## Contributing

Pull requests are welcome! Please run the following before submitting:

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

---

## License

MIT — see [LICENSE](LICENSE).
