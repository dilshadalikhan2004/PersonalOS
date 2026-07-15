# LifeOS Security Audit

Date: 2026-07-15

## Scope

Reviewed the Tauri v2 desktop app, React frontend, Rust command layer, SQLite repositories, encrypted file storage, OCR execution, Ollama loopback integration, LanceDB vector storage, and local reminder/timeline/dashboard surfaces.

## Findings Fixed

- Reduced Tauri capabilities from `core:default` to the minimum commands currently needed by the UI.
- Hardened CSP with `object-src 'none'`, `base-uri 'none'`, `form-action 'none'`, and `frame-ancestors 'none'`.
- Removed broad frontend network allowance from `connect-src`; IPC remains local-only.
- Canonicalized upload paths at the command boundary.
- Rejected relative upload paths and non-file paths.
- Rejected symlink uploads before reading document bytes.
- Fixed upload progress regression where secure storage reported a lower percentage after metadata encryption.
- Constrained LanceDB delete filters to canonical UUIDs before building the required LanceDB expression string.
- Clamped vector search limits.
- Clamped chat history reads.
- Clamped upload listing reads.
- Removed OCR executable fallback to `PATH`; LifeOS now runs only explicitly bundled OCR tools from the app data `ocr-tools` directory.

## Findings Reviewed

- Encryption uses AES-256-GCM with random 96-bit nonces.
- The master key is stored only in the OS keychain and is not serialized, cloned, logged, or returned to the UI.
- SQLite user data fields are stored as encrypted payload blobs for documents, uploads, OCR results, structured metadata, and chat history.
- SQL statements in SQLite repositories use parameter binding.
- Rust application code contains no `unsafe` blocks.
- External OCR commands are invoked without a shell, with validated OCR language identifiers, and from an application-controlled tool directory.
- Ollama access is hardcoded to `127.0.0.1:11434`; there is no remote AI endpoint configuration.
- Public command errors avoid leaking paths, database details, or cryptographic failure detail.

## Residual Risks

- LanceDB vector embeddings are stored in LanceDB for search. They are derived from document text and should be treated as sensitive local data. Full text remains encrypted in SQLite.
- OCR now requires bundled `tesseract`, `pdftotext`, and `pdftoppm` binaries in the local `ocr-tools` directory. Missing binaries fail closed instead of searching `PATH`.
- OCR currently runs against the selected local source path after upload. The upload itself is encrypted immediately, but a future hardening pass should OCR from a decrypted temporary copy of the encrypted stored file.
- The browser dev server is for development only. Production security depends on running through the Tauri app bundle with the configured CSP and capabilities.
- Local browser/webview notifications depend on OS/browser permission state and do not persist scheduled reminders after the app exits.

## Verification

- `cargo check`
- `npm.cmd run typecheck`
- `npm.cmd run lint`
- `npm.cmd run build`
