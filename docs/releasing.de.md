# Releases veröffentlichen

> Deutsche Fassung. Englisches Original: [releasing.md](releasing.md)

Releases baut GitHub Actions ([release.yml](../.github/workflows/release.yml)), sobald ein `v*`-Tag gepusht wird.

| Plattform    | Runner                | Pakete                                        |
| ------------ | --------------------- | --------------------------------------------- |
| Linux x86_64 | Ubuntu 22.04          | `.deb`, `.rpm`, `.AppImage`                   |
| Linux arm64  | Ubuntu 22.04 (arm)    | `.deb`, `.rpm`, `.AppImage`                   |
| macOS        | macOS (Apple Silicon) | universelles `.dmg` (Apple Silicon und Intel) |
| Windows x64  | Windows               | `.msi`, `-setup.exe` (NSIS)                   |

Jedes Release enthält außerdem `SHA256SUMS.txt`. Versionen vor 1.0 und Tags mit `alpha`, `beta` oder `rc` werden als Vorabversion markiert.

## Schritte

1. Die neue Version in `Cargo.toml` (`[workspace.package]`) und `ui/package.json` eintragen, dann `cargo check` ausführen, damit `Cargo.lock` nachzieht. Der Release-Job lehnt einen Tag ab, der nicht zu beiden passt.
2. Die Notizen in `docs/release-notes/v<version>.md` schreiben: zuerst Englisch, dann ein Abschnitt `## Deutsch`. Ohne diese Datei wird ein allgemeiner Text in beiden Sprachen verwendet.
3. Committen, taggen und pushen:

   ```bash
   git tag v0.1.0
   git push origin main v0.1.0
   ```

4. Wenn der Workflow fertig ist, das Release auf GitHub prüfen.

Ein manueller Lauf des Workflows (_Actions › Release › Run workflow_) baut alle Pakete und behält sie als Workflow-Artefakte, ohne etwas zu veröffentlichen; gedacht zum Testen von Änderungen an der Paketierung.

5. Flatpak und AUR auf das Release umstellen (siehe unten) und das Ergebnis committen.

## Signieren

Der Release-Workflow signiert, wenn diese Repository-Secrets existieren, und baut sonst unsignierte Pakete:

- **macOS** (Signieren und Notarisieren durch Tauri): `APPLE_CERTIFICATE` (das Developer-ID-Application-Zertifikat als Base64-`.p12`), `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` (zum Beispiel `Developer ID Application: Name (TEAMID)`) und zum Notarisieren `APPLE_ID`, `APPLE_PASSWORD` (ein app-spezifisches Passwort) und `APPLE_TEAM_ID`. Dafür ist ein kostenpflichtiges Apple-Developer-Konto nötig.
- **Windows:** `WINDOWS_CERTIFICATE` (ein Code-Signing-Zertifikat als Base64-`.pfx`) und `WINDOWS_CERTIFICATE_PASSWORD`. Der Workflow importiert es und gibt seinen Fingerabdruck an Tauri, das `.msi` und Setup mit SHA-256-Zeitstempel signiert.

## Flatpak und AUR

Beide werden aus den `.deb`-Dateien eines GitHub-Releases gebaut und folgen damit einem Release:

```bash
node tools/update-packaging.mjs 0.3.0
```

setzt Version und Prüfsummen in `packaging/flatpak/` und `packaging/aur/`. Das Flatpak-Manifest baut mit `flatpak-builder --user --install build packaging/flatpak/io.github.soulinfernode.synocompanion.yml`; es bringt die AppIndicator-Bibliotheken für das Tray-Icon mit und fragt nach dem Secret Service (Schlüsselbund), Benachrichtigungen und dem Tray. Für Flathub kommt das Manifest dort in ein eigenes Repository; für das AUR werden `PKGBUILD` und `.SRCINFO` in das Paket `syno-companion-bin` gepusht. Beides braucht ein Konto des Maintainers.

## Noch nicht

- **Automatische Updates:** geplant, sobald Releases signiert sind.
