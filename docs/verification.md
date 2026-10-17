# Verification

Version 0.1.0 was checked on Windows x64 on 17 October 2026, using Node 24.18.0, npm 11.16.0, Rust 1.99.1 and Tauri CLI 2.12.3.

## Automated checks

| Command                                                                                   | Result                                               |
| ----------------------------------------------------------------------------------------- | ---------------------------------------------------- |
| `npm run format:check`                                                                    | Passed                                               |
| `npm run lint`                                                                            | Passed, zero warnings                                |
| `npm test`                                                                                | 3 tests passed                                       |
| `npm run build`                                                                           | TypeScript and Vite production build passed          |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check`                                  | Passed                                               |
| `cargo test --locked --manifest-path src-tauri/Cargo.toml --no-default-features`          | 15 integration tests passed                          |
| `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Passed                                               |
| `npm run desktop:build`                                                                   | Windows x64 executable and NSIS installer built      |
| `npm audit`                                                                               | Zero known npm vulnerabilities after updating Vitest |

Native tests include malformed JSON, duplicate object keys, namespace conventions, missing references, local model and alias cycles, PNG corruption and dimensions, 8-bit transparency, near-opaque 16-bit alpha, oversized input, ZIP traversal and corruption, missing paths, empty and 2,000-asset projects, cancellation and report serialization.

## Desktop smoke checks

- Launched the development desktop app and the standalone production executable. The production app rendered with the development server stopped.
- Opened `broken-pack`: 7 supported assets, 1 error, 4 warnings, 1 informational finding.
- Inspected real PNG dimensions, alpha metadata and original pixels. Increased preview zoom from 400% to 800%.
- Opened a ZIP created from `valid-pack` in the standalone executable: 6 supported assets, no findings.
- Inspected the ZIP's transparent crystal PNG; the preview showed the checkerboard and reported transparent pixels.
- Rescanned the imported ZIP successfully.
- Exercised the native report-save dialog. Saving inside the source pack was refused. A new report outside the pack was created and its counts matched the UI.
- Compared SHA-256 hashes of fixture files after inspection/export: no changes.
- Checked the remapped production executable for the actual workspace and user-directory strings: absent.
- Screenshots show the desktop application with the included fixtures.
