# fbadmin

A command-line tool for managing Firebase Authentication — users, custom claims, auth action links, and emulator utilities.

## Installation

### Homebrew (macOS & Linux)

```bash
brew install NeoScript/fbadmin/fbadmin
```

### Shell installer (macOS & Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/NeoScript/firebase-admin-cli/releases/latest/download/fbadmin-installer.sh | sh
```

### PowerShell installer (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/NeoScript/firebase-admin-cli/releases/latest/download/fbadmin-installer.ps1 | iex"
```

### From source

Requires Rust 1.94+.

```bash
cargo install --path .
```

## Quick start

```bash
# Interactive setup — creates a named profile
fbadmin config init

# Or connect directly with flags / env vars
fbadmin --credentials ~/sa-key.json users list
fbadmin -e localhost:9099 emulator clear-users
export FBADMIN_PROJECT=my-project
fbadmin users count
```

## Authentication

fbadmin resolves credentials in this order:

1. `--credentials` / `FBADMIN_CREDENTIALS` — path to a service account JSON file
2. `--project` / `FBADMIN_PROJECT` — project ID using Application Default Credentials
3. `--emulator-host` / `FBADMIN_EMULATOR_HOST` — connect to the Firebase Auth emulator
4. Profile settings from config (see below)

## Configuration

Profiles store connection settings so you don't need to pass flags every time.

```bash
fbadmin config init          # Guided wizard
fbadmin config add prod --credentials ~/keys/prod-sa.json
fbadmin config add local --emulator-host localhost:9099
fbadmin config default prod  # Set the default profile
fbadmin config list          # Show all profiles
fbadmin config which         # Show resolved connection chain
fbadmin config path          # Print config file locations
```

Global config is stored by `confy` in the OS-appropriate location. A local `.fbadmin.toml` in the working directory is merged on top (field-level override).

## Commands

### Users

```bash
fbadmin users get --email user@example.com
fbadmin users get --uid abc123
fbadmin users create --email new@example.com
fbadmin users create --email new@example.com --password s3cret --display-name "Jane Doe"
fbadmin users disable --email user@example.com
fbadmin users enable --email user@example.com
fbadmin users remove --csv uids.csv          # Bulk delete from CSV
fbadmin users list --limit 50
fbadmin users list-inactive --days 90
fbadmin users count
```

Missing required arguments are prompted interactively when running in a terminal. Passwords are auto-generated if omitted.

### Custom claims

```bash
fbadmin claims get --email user@example.com
fbadmin claims merge role admin --email user@example.com
fbadmin claims merge tier 2 --email user@example.com        # Auto-detects int
fbadmin claims merge prefs '{"dark":true}' --email user@example.com  # JSON
fbadmin claims remove role --email user@example.com
fbadmin claims clear --email user@example.com
fbadmin claims find admin                     # Find all users with "admin" claim
fbadmin claims find role admin --exclusive     # Only where role is the sole claim
```

Use `--dry-run` with `merge`, `remove`, and `clear` to preview changes without writing.

### Auth action links

```bash
fbadmin links password-reset --email user@example.com
fbadmin links email-verify --email user@example.com
fbadmin links sign-in --email user@example.com
```

### Emulator

These commands only work when connected to an emulator.

```bash
fbadmin -e localhost:9099 emulator clear-users
fbadmin -e localhost:9099 emulator config
```

### Connection info

```bash
fbadmin info    # Shows resolved profile, project, credentials, and verifies connectivity
```

## Global flags


| Flag              | Short | Env var                 | Description                           |
| ----------------- | ----- | ----------------------- | ------------------------------------- |
| `--profile`       | `-p`  | `FBADMIN_PROFILE`       | Named profile from config             |
| `--project`       |       | `FBADMIN_PROJECT`       | Firebase project ID                   |
| `--credentials`   | `-c`  | `FBADMIN_CREDENTIALS`   | Path to service account JSON          |
| `--emulator-host` | `-e`  | `FBADMIN_EMULATOR_HOST` | Emulator host:port                    |
| `--format`        | `-f`  |                         | Output format: `table`, `json`, `csv` |
| `--dry-run`       |       |                         | Preview destructive operations        |
| `--yes`           | `-y`  |                         | Skip confirmation prompts             |
| `--verbose`       | `-v`  |                         | Increase verbosity (`-vv`, `-vvv`)    |


## Output formats

```bash
fbadmin users list -f table   # Human-readable table (default)
fbadmin users list -f json    # NDJSON — one JSON object per line
fbadmin users list -f csv     # CSV with headers
fbadmin claims get --email user@example.com -f json  # Single record as JSON
```

## License

AGPL-3.0-only — see [LICENSE](LICENSE).