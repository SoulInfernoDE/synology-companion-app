# Architecture

> English original. German version: [architecture.de.md](architecture.de.md)

## Overview

```
┌──────────────── Desktop (Linux, macOS, Windows) ────────────────┐
│  UI: Svelte 5 in the system WebView, skins, Fluent translations  │
│        │ Tauri commands / events                                 │
│  Core (Rust)                                                     │
│   ├─ discovery    mDNS/Bonjour, Wake-on-LAN                      │
│   ├─ dsm          DSM Web API client (HTTPS, session, 2FA)       │
│   ├─ ssh          keys, host key pinning, sudo via keychain      │
│   ├─ recipes      manifest, plan, backup, apply, verify, restore │
│   ├─ journal      what was changed, on which NAS, when           │
│   └─ secrets      system keychain                                │
└──────────────────────────┬───────────────────────────────────────┘
          HTTPS (5001)     │      SSH (22 or custom port)
┌──────────────────────────┴───────────────────────────────────────┐
│  Synology NAS, DSM 7.3+                                           │
└───────────────────────────────────────────────────────────────────┘
```

**Why Tauri:** one code base for all three systems, native Rust for network, SSH and keychain, the system WebView instead of a bundled browser (small packages), and the same stack as [overLay](https://github.com/SoulInfernoDE/overLay).

**Two channels to the NAS.** Everything DSM offers is done through the **Web API**: it needs no SSH, works for every user and survives DSM updates. **SSH** is used only where the API ends: recipes, diagnostics, Nextcloud's `occ`. The UI says which channel an action uses.

## Repository layout

| Path                         | Contents                                                 |
| ---------------------------- | -------------------------------------------------------- |
| `crates/companion-app`       | Tauri app: windows, tray, commands                       |
| `crates/companion-dsm`       | DSM Web API client with certificate pinning              |
| `crates/companion-discovery` | Finds NAS on the local network (mDNS/Bonjour)            |
| `crates/companion-ssh`       | SSH, keys, sudo _(planned)_                              |
| `crates/companion-recipes`   | Recipe engine and journal _(planned)_                    |
| `ui/`                        | User interface (Svelte 5, TypeScript, Vite)              |
| `locales/`                   | Fluent catalogs, shared by Rust and UI (`en` base, `de`) |
| `recipes/`                   | Recipe manifests _(planned)_                             |
| `assets/branding/`           | Icon and logos                                           |

## Signing in

1. The app asks DSM which API versions it offers (`SYNO.API.Info`), then signs in with `SYNO.API.Auth`. Every call is an HTTPS form POST, so neither password nor session ID ends up in a URL.
2. **Certificates.** A certificate the system trusts (for example Let's Encrypt on DSM) is accepted as usual. DSM's own self-signed certificate is not: the app shows its SHA-256 fingerprint, and once the user trusts it, that exact certificate is pinned for this NAS. A different certificate later is refused with a warning, unless the system trusts it.
3. **Two-step verification.** When DSM asks for a code, the app asks the user. With _remember this computer_ DSM issues a device token; the next sign-in sends it instead of a code.
4. **Keychain.** The device token, and the password only if the user asks for it, are stored in the system keychain under "Syno Companion", one entry per NAS and account. The list of NAS (`devices.json` in the app's config folder) holds no secrets.
5. When DSM ends an idle session, the app signs in again quietly if the keychain has the password; otherwise it returns to the start screen.

## Skins

A skin is a set of CSS custom properties plus a few shape rules (corner radius, font, control height, focus style). The UI sets `data-skin="linux|macos|windows"` and `data-scheme="light|dark"` on `<html>`. By default the skin follows the operating system the app runs on and the scheme follows the system setting; both can be overridden in the settings. Components only use the tokens, never fixed colors, so a new skin is one CSS block.

## Translations

All UI text comes from Fluent catalogs in `locales/<locale>/*.ftl`. English is the base and the fallback; a missing German message shows in English, never as a raw ID. Proper names (Syno Companion, Synology, DSM, SSH, HTTPS) may appear directly.

## Recipes

A recipe wraps a community script or a set of commands so that a regular user can run it safely. Example (format not final):

```toml
id = "third-party-drives"
name = "recipe-third-party-drives"          # Fluent message ID
source = "https://github.com/007revad/Synology_HDD_db"
version = "v3.6.139"
sha256 = "…"
license = "MIT"
dsm = ">=7.3"
expert = true
survives-update = false                       # re-apply after DSM updates

[steps]
check   = "syno_hdd_db.sh --showedits"  # read-only
backup  = "builtin:backup /var/lib/disk-compatibility"
apply   = "syno_hdd_db.sh --noupdate"
verify  = "syno_hdd_db.sh --showedits"
restore = "syno_hdd_db.sh --restore"
```

Rules for every recipe:

1. The script is downloaded from the pinned release and checked against `sha256`; a mismatch stops everything.
2. `check` and `plan` change nothing; their output is shown before the user confirms.
3. `backup` runs before `apply`; `restore` must undo `apply` completely.
4. Every run is written to the journal (time, NAS, DSM build, recipe version, result).
5. Recipes with `survives-update = false` are listed by the _after the update_ assistant.
