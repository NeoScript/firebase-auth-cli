# fire-auth

A command-line tool for managing Firebase Authentication — users, custom claims, auth action links, and emulator utilities.

## Installation

### Homebrew (macOS & Linux)

```bash
brew install NeoScript/fire-auth/firebase-auth-cli
```

### Shell installer (macOS & Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/NeoScript/firebase-auth-cli/releases/latest/download/firebase-auth-cli-installer.sh | sh
```

### PowerShell installer (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/NeoScript/firebase-auth-cli/releases/latest/download/firebase-auth-cli-installer.ps1 | iex"
```

### From source

Requires Rust 1.94+.

```bash
cargo install --path .
```

## Quick start

```bash
# Interactive setup — creates a named profile
fire-auth config init

# Or connect directly with flags / env vars
fire-auth --credentials ~/sa-key.json users list
fire-auth -e localhost:9099 emulator clear-users
export FIRE_AUTH_PROJECT=my-project
fire-auth users count
```

## Authentication

fire-auth resolves credentials in this order:

1. `--credentials` / `FIRE_AUTH_CREDENTIALS` — path to a service account JSON file
2. `--project` / `FIRE_AUTH_PROJECT` — project ID using Application Default Credentials
3. `--emulator-host` / `FIRE_AUTH_EMULATOR_HOST` — connect to the Firebase Auth emulator
4. Profile settings from config (see below)

## Configuration

Profiles store connection settings so you don't need to pass flags every time.

```bash
fire-auth config init          # Guided wizard
fire-auth config add prod --credentials ~/keys/prod-sa.json
fire-auth config add local --emulator-host localhost:9099
fire-auth config default prod  # Set the default profile
fire-auth config list          # Show all profiles
fire-auth config which         # Show resolved connection chain
fire-auth config path          # Print config file locations
```

Global config is stored by `confy` in the OS-appropriate location. A local `.fire-auth.toml` in the working directory is merged on top (field-level override).

## Commands

### Users

```bash
fire-auth users get --email user@example.com
fire-auth users get --uid abc123
fire-auth users create --email new@example.com
fire-auth users create --email new@example.com --password s3cret --display-name "Jane Doe"
fire-auth users disable --email user@example.com
fire-auth users enable --email user@example.com
fire-auth users remove --csv uids.csv          # Bulk delete from CSV
fire-auth users list --limit 50
fire-auth users list-inactive --days 90
fire-auth users count
```

Missing required arguments are prompted interactively when running in a terminal. Passwords are auto-generated if omitted.

### Custom claims

```bash
fire-auth claims get --email user@example.com
fire-auth claims merge role admin --email user@example.com
fire-auth claims merge tier 2 --email user@example.com        # Auto-detects int
fire-auth claims merge prefs '{"dark":true}' --email user@example.com  # JSON
fire-auth claims remove role --email user@example.com
fire-auth claims clear --email user@example.com
fire-auth claims find admin                     # Find all users with "admin" claim
fire-auth claims find role admin --exclusive     # Only where role is the sole claim
```

Use `--dry-run` with `merge`, `remove`, and `clear` to preview changes without writing.

### Auth action links

```bash
fire-auth links password-reset --email user@example.com
fire-auth links email-verify --email user@example.com
fire-auth links sign-in --email user@example.com
```

### Emulator

These commands only work when connected to an emulator.

```bash
fire-auth -e localhost:9099 emulator clear-users
fire-auth -e localhost:9099 emulator config
```

### Connection info

```bash
fire-auth info    # Shows resolved profile, project, credentials, and verifies connectivity
```

## Global flags


| Flag              | Short | Env var                  | Description                           |
| ----------------- | ----- | ------------------------ | ------------------------------------- |
| `--profile`       | `-p`  | `FIRE_AUTH_PROFILE`      | Named profile from config             |
| `--project`       |       | `FIRE_AUTH_PROJECT`      | Firebase project ID                   |
| `--credentials`   | `-c`  | `FIRE_AUTH_CREDENTIALS`  | Path to service account JSON          |
| `--emulator-host` | `-e`  | `FIRE_AUTH_EMULATOR_HOST`| Emulator host:port                    |
| `--format`        | `-f`  |                          | Output format: `table`, `json`, `csv` |
| `--dry-run`       |       |                          | Preview destructive operations        |
| `--yes`           | `-y`  |                          | Skip confirmation prompts             |
| `--verbose`       | `-v`  |                          | Increase verbosity (`-vv`, `-vvv`)    |


## Output formats

```bash
fire-auth users list -f table   # Human-readable table (default)
fire-auth users list -f json    # NDJSON — one JSON object per line
fire-auth users list -f csv     # CSV with headers
fire-auth claims get --email user@example.com -f json  # Single record as JSON
```

## License

AGPL-3.0-only — see [LICENSE](LICENSE).
