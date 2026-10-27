# Releasing

> English original. German version: [releasing.de.md](releasing.de.md)

Releases are built by GitHub Actions ([release.yml](../.github/workflows/release.yml)) when a `v*` tag is pushed.

| Platform     | Runner                | Packages                                   |
| ------------ | --------------------- | ------------------------------------------ |
| Linux x86_64 | Ubuntu 22.04          | `.deb`, `.rpm`, `.AppImage`                |
| Linux arm64  | Ubuntu 22.04 (arm)    | `.deb`, `.rpm`, `.AppImage`                |
| macOS        | macOS (Apple silicon) | universal `.dmg` (Apple silicon and Intel) |
| Windows x64  | Windows               | `.msi`, `-setup.exe` (NSIS)                |

Every release also carries `SHA256SUMS.txt`. Versions before 1.0 and tags with `alpha`, `beta` or `rc` are marked as pre-releases.

## Steps

1. Set the new version in `Cargo.toml` (`[workspace.package]`) and `ui/package.json`, then run `cargo check` so `Cargo.lock` follows. The release job refuses a tag that does not match both.
2. Write the notes in `docs/release-notes/v<version>.md`: English first, then a `## Deutsch` section. Without this file a generic text in both languages is used.
3. Commit, tag and push:

   ```bash
   git tag v0.1.0
   git push origin main v0.1.0
   ```

4. Check the release on GitHub when the workflow is done.

A manual run of the workflow (_Actions › Release › Run workflow_) builds all packages and keeps them as workflow artifacts without publishing anything; use it to test packaging changes.

5. Point Flatpak and AUR at the release (see below) and commit the result.

## Signing

The release workflow signs when these repository secrets exist and builds unsigned packages otherwise:

- **macOS** (signing and notarisation by Tauri): `APPLE_CERTIFICATE` (the Developer ID Application certificate as base64 `.p12`), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` (for example `Developer ID Application: Name (TEAMID)`), and for notarisation `APPLE_ID`, `APPLE_PASSWORD` (an app-specific password) and `APPLE_TEAM_ID`. This needs a paid Apple Developer account.
- **Windows:** `WINDOWS_CERTIFICATE` (a code signing certificate as base64 `.pfx`) and `WINDOWS_CERTIFICATE_PASSWORD`. The workflow imports it and passes its thumbprint to Tauri, which signs the `.msi` and the setup with a SHA-256 timestamp.

## Flatpak and AUR

Both are built from the `.deb` files of a GitHub release, so they follow a release:

```bash
node tools/update-packaging.mjs 0.3.0
```

sets version and checksums in `packaging/flatpak/` and `packaging/aur/`. The Flatpak manifest builds with `flatpak-builder --user --install build packaging/flatpak/io.github.soulinfernode.synocompanion.yml`; it adds the AppIndicator libraries for the tray icon and asks for the Secret Service (keychain), notifications and the tray. For Flathub the manifest goes into a repository of its own there; for the AUR, `PKGBUILD` and `.SRCINFO` are pushed to the `syno-companion-bin` package. Both need an account of the maintainer.

## Not yet

- **Automatic updates:** planned once releases are signed.
