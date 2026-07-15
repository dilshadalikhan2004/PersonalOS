# LifeOS

Privacy-first, local-only personal AI life organizer.

## Prerequisites

- Node.js 20+
- Rust stable with Cargo
- Tauri v2 platform prerequisites: https://v2.tauri.app/start/prerequisites/

## Setup

```powershell
npm.cmd install
npm.cmd run tauri dev
```

All application data and future AI integrations must remain on the device. No user content may be transmitted externally.

## Architecture

- `src/ui`: React presentation layer and shadcn/ui components
- `src/application`: frontend use cases and ports
- `src/domain`: frontend business concepts with no framework dependencies
- `src/infrastructure`: Tauri adapters and external-system implementations
- `src-tauri/src`: Rust clean architecture layers

## Local encrypted document storage

Document CRUD is implemented in the Rust backend. Each document payload (title, body, and timestamps) is serialized and protected with AES-256-GCM before being atomically written to the app-data `documents/` directory. SQLite is an opaque local index of random document IDs and random encrypted-file names; it contains no readable document content or timestamps.

The 256-bit data-encryption key is generated with the operating system RNG and stored only in the operating system keychain (Windows Credential Manager, macOS Keychain, or Linux Secret Service). It is never serialized, logged, returned through Tauri commands, or stored in the application directory. If the OS keychain is unavailable, LifeOS refuses to open encrypted storage.

## Offline OCR

LifeOS runs OCR only through local executables: Tesseract for images and Poppler (`pdftotext`/`pdftoppm`) for PDF text detection and rasterization. PDF files with meaningful embedded text are extracted directly; otherwise, their locally rendered pages are processed by Tesseract. OCR output, source type, language selection, and timestamp are encrypted before SQLite persistence.

Distribution builds must bundle Tesseract, Poppler, and the selected local `tessdata` language packs in the app-data `ocr-tools/` directory. The app never downloads language packs or sends files to an OCR service. Supported language specifications are Tesseract identifiers joined with `+`, such as `eng+hin`.

## Local AI classification

After local text extraction, LifeOS sends the text only to Ollama's fixed loopback address, `127.0.0.1:11434`. The endpoint is not configurable and cloud Ollama endpoints are rejected by design. A pre-installed local `qwen2.5:3b-instruct` model returns schema-constrained document metadata: title, type, dates, expiry, contacts, addresses, companies, amounts, and important identifiers. The response is normalized, encrypted with AES-256-GCM, and saved in SQLite; no original text or structured metadata is written in plaintext.
