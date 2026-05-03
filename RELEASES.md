# v0.1.0

Initial release of fbadmin — Firebase Auth administration CLI.

## Features

- **User management**: get, create, disable, enable, remove (single or bulk CSV), list, list-inactive, count
- **Custom claims**: get, merge, remove, clear, find users by claim
- **Auth action links**: generate password-reset, email-verification, and sign-in links
- **Emulator support**: clear-users, config — works with Firebase Auth emulator
- **Configuration profiles**: named profiles with project, credentials, and emulator settings
- **Output formats**: table (default), JSON (NDJSON), CSV
- **Rich CLI**: colored output, progress spinners, `--verbose` logging, `--dry-run` mode
