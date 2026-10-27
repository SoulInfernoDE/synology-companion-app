# Architecture

> English original. German version: [architecture.de.md](architecture.de.md)

## Overview

```
┌──────────────── Desktop (Linux, macOS, Windows) ────────────────┐
│  UI: Svelte 5 in the system WebView, skins, Fluent translations  │
│        │ Tauri commands / events                                 │
│  Core (Rust)                                                     │
│   ├─ discovery    mDNS/Bonjour, Synology Assistant, Wake-on-LAN │
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

| Path                         | Contents                                                     |
| ---------------------------- | ------------------------------------------------------------ |
| `crates/companion-app`       | Tauri app: windows, tray, commands                           |
| `crates/companion-dsm`       | DSM Web API client with certificate pinning                  |
| `crates/companion-discovery` | Finds NAS on the local network (Bonjour, Synology Assistant) |
| `crates/companion-ssh`       | SSH, keys, sudo _(planned)_                                  |
| `crates/companion-recipes`   | Recipe engine and journal _(planned)_                        |
| `ui/`                        | User interface (Svelte 5, TypeScript, Vite)                  |
| `locales/`                   | Fluent catalogs, shared by Rust and UI (`en` base, `de`)     |
| `recipes/`                   | Recipe manifests _(planned)_                                 |
| `assets/branding/`           | Icon and logos                                               |

## Finding NAS

Two searches run at the same time for three seconds:

- **Bonjour/mDNS:** DSM announces `_http._tcp` with `vendor=Synology`, model, DSM version and HTTPS port, if Bonjour is on in DSM.
- **Synology Assistant's protocol:** a query broadcast to UDP port 9999, answered by DSM's `findhostd` with host name, IP address, model, DSM version and ports, whether Bonjour is on or not. A packet is the magic `12 34 56 78 'SYNO'` followed by records of one byte ID, one byte length and the value. The layout follows public analyses of `findhostd` and [syno_discover.py](https://github.com/007revad/Syno_Toolbox) by 007revad (MIT). The query names a locally administered sender MAC, because findhostd ignores queries from Synology's own MAC prefixes; it is new for every query and the query is sent three times, because findhostd stays silent for a while towards a sender it has just answered and UDP may lose packets. The serial number in the answers is not kept.

Answers are merged by IP address. On Linux and Windows a firewall may need to let UDP port 9999 in; on macOS the system asks once for access to the local network.

## Signing in

1. The app asks DSM which API versions it offers (`SYNO.API.Info`), then signs in with `SYNO.API.Auth`. Every call is an HTTPS form POST, so neither password nor session ID ends up in a URL.
2. **Certificates.** A certificate the system trusts (for example Let's Encrypt on DSM) is accepted as usual. DSM's own self-signed certificate is not: the app shows its SHA-256 fingerprint, and once the user trusts it, that exact certificate is pinned for this NAS. A different certificate later is refused with a warning, unless the system trusts it.
3. **Two-step verification.** When DSM asks for a code, the app asks the user. With _remember this computer_ DSM issues a device token; the next sign-in sends it instead of a code.
4. **Keychain.** The device token, and the password only if the user asks for it, are stored in the system keychain under "Syno Companion", one entry per NAS and account. The list of NAS (`devices.json` in the app's config folder) holds no secrets.
5. When DSM ends an idle session, the app signs in again quietly if the keychain has the password; otherwise it returns to the start screen.

## First-time setup

Until DSM's welcome wizard has run, DSM hands an administrator session to whoever opens the web interface. Before asking for a password, the app checks this: it opens the start page and asks `SYNO.Core.QuickStart.Info check_permission`. A NAS that is set up hands out no session and answers "no session" (119). (The `configured` flag in `SYNO.Core.Desktop.SessionData` is no help: it stays `false` after setup, too.) For a NAS that is not set up, it reads the password rules, reserved names and time zones, shows its own form and, after the user confirms, sends one `SYNO.Entry.Request` that stops at the first error, in this order:

1. block addresses after failed sign-ins, set the device name;
2. create the user's account and add it to `administrators`;
3. stop the registration with find.synology.com;
4. only then give the built-in `admin` account a random password and disable it.

Afterwards the app signs in with the new account and sets updates, time zone, usage statistics (off) and hides DSM's welcome wizard. These later settings are optional; any DSM refuses are listed, the setup still counts as done.

## Managing a NAS

- **Updates:** DSM offers its next version through `SYNO.Core.Upgrade.Server check`. For packages there is no single call: the app compares the installed versions (`SYNO.Core.Package list`) with Package Center's catalogue (`SYNO.Core.Package.Server list`), the way Package Center does. The check asks Synology's servers, so the app runs it on opening and then every half hour.
- **Power:** restart and shut down use `SYNO.Core.System reboot`/`shutdown` without `force`, so DSM refuses while a task must not be interrupted; the beeper is silenced with `SYNO.Core.Hardware.BeepControl stop_beep`.
- **Wake-on-LAN:** DSM's Web API reports no MAC address, but Bonjour and findhostd do. The app keeps the MAC of a saved NAS from discovery and sends the magic packet to the whole network and to the NAS's last /24 network, on ports 9 and 7.
- **Shared folders** come from `SYNO.Core.Share list`; hidden ones stay hidden. They open over SMB in the system's file manager: `smb://` URLs on Linux and macOS, `\\host\share` in Explorer on Windows. DSM serves the folder `homes` as `home`, the user's own home folder.

## Tray and notifications

A background task asks the active NAS every minute, also while the window is hidden, and keeps what it saw: the tray shows "name: state", and a notification tells when a volume or drive becomes worse than before, when the NAS stops or starts answering, when everything is healthy again and, checked every six hours, when DSM offers a new version (once per version). Tray menu and notifications use the same Fluent catalogs as the interface, embedded into the binary; the interface hands its language to the core. Whether closing the window keeps the app in the tray, and whether notifications are shown, is kept in `preferences.json` next to the list of NAS. Keeping the app in the tray is off by default, because some Linux desktops show no tray icons.

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
