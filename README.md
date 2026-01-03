# Secure File Sharing REST Server (Rust) + CLI

Course project: a RESTful file-sharing service with:
- Axum-based HTTP API + SQLite for persistence
- Auth with JWT access tokens + refresh tokens (stored in the DB)
- File upload/download with access control (private/shared) and optional public files
- Share permissions and revoke permissions
- CLI client (`sfs`) to demo the main flows (stores tokens locally)
---

## Requirements
- Rust toolchain (tested on nightly): `cargo`, `rustc`
- SQLite (used for persistence)
- Optional: `sqlite3` CLI (only for manual inspection of `data/app.db`)
---

## Server setup

### 1) Configure env
Create a `.env` in the repo root:
```bash
JWT_SECRET=change_me_to_a_long_random_string
```
The server loads `.env` at startup via `dotenvy`.

### 2) Run the server (dev)
From the repo root:
```bash
cargo run
```
(Leave this running; use a second terminal for curl/CLI commands.)

Server listens on:
```bash
0.0.0.0:8080
```
(Open in browser as `http://localhost:8080`.)

On first run it creates (if missing):
- `data/`
- `data/app.db`

On first upload it creates (if missing):
- `data/uploads/`

### 3) Build server (release)
Use this instead of step 2 if you want an optimized binary:
```bash
cargo build --release
./target/release/secure_file_server
```
---

## Client setup (`sfs`)

You have 2 ways to run the client.

### Option A (recommended): install `sfs` into your Cargo bin
From the repo root:
```bash
cargo install --path client --bin sfs --force
```
`cargo install` builds a release binary and places it in `~/.cargo/bin` (make sure that's on your `PATH`).

Now you can run `sfs` from anywhere:
```bash
sfs --help
sfs health
```

### Option B: run without installing
From the repo root:
```bash
cargo run --manifest-path client/Cargo.toml -- --help
cargo run --manifest-path client/Cargo.toml -- health
```

Build client (release binary)
```bash
cargo build --manifest-path client/Cargo.toml --release
./client/target/release/sfs --help
```

### Server base URL
By default the client targets `http://localhost:8080`. You can override it with:
```bash
sfs --base http://localhost:8080 health
```

### Where tokens are stored
The CLI stores tokens in your OS config directory using `directories::ProjectDirs`.

Example on macOS:
```text
~/Library/Application Support/com.programming-3.sfs/tokens.json
```

(Windows/Linux use different base directories, but the app folder name is the same: `com.programming-3.sfs`.)

---

## Quick demo flow (copy/paste)

### Terminal A: run the server from the repo root
From repo root:

```bash
rm -rf data
cargo run
```

### Terminal B: run the commands below
Note: if you want to do the reset step, stop the server first (or do the reset before starting Terminal A).

```bash
# Health
sfs health

# Register users
sfs register demoA 'demoPass123!'
sfs logout
sfs register demoB 'demoPass123!'
sfs logout

# Login as demoA
sfs login demoA 'demoPass123!'
sfs me # prints HTTP 200 when authenticated

# Upload a PRIVATE file (note the printed file_id)
echo "hello private $(date)" > demo_private.txt
sfs upload demo_private.txt
# Example output: Uploaded file_id=1 ...

# Download it (replace 1 with your actual id)
sfs download 1 --out downloaded_private.txt
cat downloaded_private.txt
echo

# Upload a PUBLIC file (note the printed file_id)
echo "hello public $(date)" > demo_public.txt
sfs upload demo_public.txt --public
# Example output: Uploaded file_id=2 ...

# Public download (no login required) (replace 2 with your actual id)
sfs logout
sfs public-download 2 --out downloaded_public.txt
cat downloaded_public.txt
echo

# List files (demoA)
sfs login demoA 'demoPass123!'
sfs list
sfs files

# Share + revoke
# On a fresh DB after the reset above, demoA will be user_id=1 and demoB will be user_id=2.
# If you did NOT reset the DB, demoB may not be user_id=2.
# Use your private file_id from the upload step above.
sfs share 1 2
sfs revoke-user 1 2

# Cleanup local text files
sfs logout
rm -f demo_private.txt demo_public.txt downloaded_private.txt downloaded_public.txt
echo "DONE"
```

---

## API endpoints

### Public (no auth)
- `GET /health` — health check
- `POST /register` — create a user
- `POST /login` — returns access + refresh tokens
- `GET /token/refresh` — requires `Authorization: Bearer <refresh_token>`; rotates token and returns new tokens
- `GET /file/public/:id` — download a public file by id

### Protected (JWT required: `Authorization: Bearer <access_token>`)
- `GET /me` — return current user info
- `POST /file/upload` — multipart upload (`file`, optional `is_public` field)
- `GET /file/:id` — download a file you own or that was shared with you
- `GET /files` — list files visible to you (owned + shared)
- `POST /file/:id/share` — share a file you own with another user (JSON: `{"user_id": <id>}`)
- `DELETE /file/:id/share/:permission_id` — revoke a share by permission id (owner-only)
- `DELETE /file/:id/share/user/:user_id` — revoke a share for a specific user id (owner-only)

---

## Manual DB inspection
Run these from the repo root (requires `sqlite3`):

```bash
sqlite3 data/app.db "SELECT id, username, created_at, datetime(created_at,'unixepoch','localtime') FROM users ORDER BY id;"
sqlite3 data/app.db "SELECT id, filename, owner_id, is_public, uploaded_at, datetime(uploaded_at,'unixepoch','localtime') FROM files ORDER BY id;"
sqlite3 data/app.db "SELECT id, file_id, user_id, permission_type FROM permissions ORDER BY id;"
sqlite3 data/app.db "SELECT token, user_id, created_at, revoked_at, replaced_by FROM refresh_tokens ORDER BY created_at DESC;"
```

Note: if you share a file and then revoke it, the permissions table may be empty afterward (this is expected).

---

## Notes
- Uploaded files are stored on disk as `data/uploads/<file_id>.bin`.
- File metadata, sharing permissions, and refresh tokens are stored in SQLite.
- Public download works only for `is_public = 1`.
- Max upload size is 10 MB.

---

## Demo video

- [Watch the demo (720p)](https://github.com/user-attachments/assets/3dd86662-0fe8-42b5-b3d8-54d28160e06a)
