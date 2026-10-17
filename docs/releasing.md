# Release checklist

1. Use a clean Windows 10/11 x64 environment with Node 24, stable Rust, Visual C++ Build Tools and WebView2.
2. Run `npm ci`, `npm run format:check`, `npm run lint`, `npm test`, `npm run build`.
3. Run `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo test --locked --manifest-path src-tauri/Cargo.toml --no-default-features`, and `cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`.
4. Set the same version in `package.json`, `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`. Update both lockfiles.
5. Run `npm run desktop:build`. This wrapper remaps workspace and home-directory source paths out of compiled diagnostics. The NSIS installer is written under `src-tauri/target/release/bundle/nsis/`. The manual GitHub packaging workflow builds an artifact without publishing a release.
6. Install and uninstall on clean Windows 10 and 11 machines. Test a directory and ZIP, transparent PNG preview and zoom, invalid JSON diagnostics, local and vanilla dependencies, cancellation, rescanning, and report export. Verify that source assets remain unchanged.
7. Check keyboard navigation, minimum-size layout, high DPI, inaccessible files, corrupt archives and large imports. Build cross-platform binaries separately before claiming support for another platform.
8. Sign distributable artifacts when signing credentials are available. Unsigned builds can trigger Windows reputation prompts. No signing credentials are bundled.
9. Publish only after installer testing. Include checksums, supported architectures, validation scope and known limitations in release notes. Do not represent a locally built installer as broadly tested.

The default installer can bootstrap WebView2 during installation if missing. The application itself works offline after installation. For offline installation on a clean machine, install Microsoft's WebView2 Evergreen Runtime beforehand or deliberately configure Tauri's offline installer option and test the larger artifact.
