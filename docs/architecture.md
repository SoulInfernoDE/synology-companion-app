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
| `crates/companion-mockdsm`   | `mock-dsm`, a stand-in DSM to try the app without a NAS      |
| `crates/companion-ssh`       | SSH, keys, sudo, diagnostics, packages, containers           |
| `crates/companion-nextcloud` | Nextcloud: detection, compatibility, guided update           |
| `crates/companion-recipes`   | Recipe engine and journal                                    |
| `ui/`                        | User interface (Svelte 5, TypeScript, Vite)                  |
| `locales/`                   | Fluent catalogs, shared by Rust and UI (`en` base, `de`)     |
| `recipes/`                   | Recipe manifests                                             |
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
- **Packages:** the catalogue merges the installed list (`SYNO.Core.Package list`) with Package Center's offer (`SYNO.Core.Package.Server list` with `blloadothers`, so added sources such as SynoCommunity are included). Package sources are managed with `SYNO.Core.Package.Feed`, whose `list` parameter DSM expects as JSON text inside a JSON string. Adding SynoCommunity needs the trust level "Synology Inc. and trusted publishers" (`SYNO.Core.Package.Setting trust_level=1`), which the app sets only when the user leaves the box ticked. Installing and updating run over SSH as root: Synology's packages with `synopkg install_from_server`, others are downloaded on the NAS, checked against the MD5 the source publishes and installed with `synopkg install`. Missing dependencies are installed first. Packages with an install wizard take its defaults.
- **Containers** (over SSH, as root, since Docker needs root on DSM): the app lists every container with its image tag, state and Compose project (labels `com.docker.compose.project` and `…working_dir`). _Check for updates_ runs `docker pull` for every tag and marks containers whose image is no longer the one the tag names. A project is updated with `docker compose pull` and `docker compose up -d` in its folder, so volumes and settings stay as the project defines them. Single containers keep their settings only in Container Manager; the app points to its _Reset_ action instead of guessing the settings.
- **Power:** restart and shut down use `SYNO.Core.System reboot`/`shutdown` without `force`, so DSM refuses while a task must not be interrupted; the beeper is silenced with `SYNO.Core.Hardware.BeepControl stop_beep`.
- **Wake-on-LAN:** DSM's Web API reports no MAC address, but Bonjour and findhostd do. The app keeps the MAC of a saved NAS from discovery and sends the magic packet to the whole network and to the NAS's last /24 network, on ports 9 and 7.
- **Shared folders** come from `SYNO.Core.Share list`; hidden ones stay hidden. They open over SMB in the system's file manager: `smb://` URLs on Linux and macOS, `\\host\share` in Explorer on Windows. DSM serves the folder `homes` as `home`, the user's own home folder.

## SSH

The SSH client is [russh](https://github.com/Eugeny/russh) (pure Rust, Apache-2.0), so no `ssh` program is needed on the computer.

1. **Service.** `SYNO.Core.Terminal get` tells whether SSH is on and on which port; after the user agrees, `set` turns it on. Key sign-in needs DSM's user home service (`SYNO.Core.User.Home`), because `authorized_keys` lives in the home folder; the assistant warns when it is off. Only administrators may use SSH on DSM.
2. **Host key.** Like the HTTPS certificate: the first connection shows the SHA-256 fingerprint, the user trusts it, and it is pinned in `devices.json`. A different key later is refused with a warning.
3. **Key.** The app makes an ed25519 key of its own for each NAS and account and signs in once with the DSM password (password or keyboard-interactive) to append the public key to `~/.ssh/authorized_keys`, setting `.ssh` to 700, the file to 600 and removing write access for others from the home folder, as sshd's `StrictModes` requires. It then signs in with the key to check it works, and only then stores the private key in the keychain (entry `ssh:account@nas`). _Remove key_ deletes the line on the NAS and the keychain entry.
4. **sudo.** Commands that need root run as `sudo -k -S -p '' -- sh -c '…'`; the password goes to standard input, so it never appears in a process list or shell history. It comes from the keychain if the user let the app remember it, otherwise the user types it.
5. **Diagnostics** collect in one round trip: `/etc.defaults/VERSION`, `uname`, `io_uring_setup` in `/proc/kallsyms` plus `kernel.io_uring_disabled`, the driver and speed of each network interface from sysfs, and `docker --version`. Memory modules come from `dmidecode -t 17` and need sudo.
6. **Terminal** (expert mode): a PTY shell (`xterm-256color`) whose output streams to [xterm.js](https://xtermjs.org/) (MIT) through a Tauri channel; xterm.js loads only when a terminal opens.

## Tray and notifications

The app can be signed in to several NAS at once; one of them is current and shown, the others stay signed in. A background task asks every signed-in NAS every minute, also while the window is hidden, and keeps what it saw: the tray shows "name: state", and a notification tells when a volume or drive becomes worse than before, when the NAS stops or starts answering, when everything is healthy again and, checked every six hours, when DSM offers a new version (once per version). Tray menu and notifications use the same Fluent catalogs as the interface, embedded into the binary; the interface hands its language to the core. Whether closing the window keeps the app in the tray, and whether notifications are shown, is kept in `preferences.json` next to the list of NAS. Keeping the app in the tray is off by default, because some Linux desktops show no tray icons.

## Skins

A skin is a set of CSS custom properties plus a few shape rules (corner radius, font, control height, focus style). The UI sets `data-skin="linux|macos|windows"` and `data-scheme="light|dark"` on `<html>`. By default the skin follows the operating system the app runs on and the scheme follows the system setting; both can be overridden in the settings. Components only use the tokens, never fixed colors, so a new skin is one CSS block.

## Translations

All UI text comes from Fluent catalogs in `locales/<locale>/*.ftl`. English is the base and the fallback; a missing German message shows in English, never as a raw ID. Proper names (Syno Companion, Synology, DSM, SSH, HTTPS) may appear directly.

## Recipes

A recipe wraps a community script so that it can be run safely. Recipes live in `recipes/*.toml` and are built into the app (crate `companion-recipes`); they appear in the _Extensions_ section in expert mode. The first one:

```toml
id = "third-party-drives"
source = "https://github.com/007revad/Synology_HDD_db"
author = "007revad"
license = "MIT"
version = "v3.6.139"
download = "https://raw.githubusercontent.com/007revad/Synology_HDD_db/v3.6.139/syno_hdd_db.sh"
file = "syno_hdd_db.sh"
sha256 = "e8a392b9…"
dsm = "7.0"                    # oldest DSM version
survives-update = false        # a DSM update can undo it
backup = ["/etc.defaults/synoinfo.conf", "/etc/synoinfo.conf", "/var/lib/disk-compatibility"]

[steps]
check = "ls /var/lib/disk-compatibility/*.db.bak >/dev/null 2>&1 || grep -q '^drive_db_test_url=' /etc/synoinfo.conf"
apply = "bash {script} -e {options}"
restore = "bash {script} -e --restore"

[[options]]
id = "noupdate"
flag = "--noupdate"
default = true
```

Rules for every recipe:

1. `check` changes nothing and runs without root; exit status 0 means applied. The app shows it as the recipe's state, together with whether the DSM version is new enough.
2. Before anything runs, the app shows the plan: the file and version it downloads, the paths it backs up and the exact command with the chosen options.
3. The script is downloaded on the NAS into `~/.syno-companion/recipes/<id>/<version>/` and checked against `sha256`; a mismatch stops everything. It is checked once more right before it runs as root.
4. As root: the `backup` paths are saved to `~/.syno-companion/backups/<id>-<time>.tar.gz` (owned by the account), then `apply` runs; `check` tells whether it worked. `restore` undoes `apply` with the script's own restore step.
5. Every run is appended to `~/.syno-companion/journal.jsonl` on the NAS (time, recipe and version, action, options, result, DSM version and build, backup, the computer that ran it), so every computer sees the same journal; the app can export it as JSON.
6. Recipes with `survives-update = false` are listed by the _after the update_ assistant (v0.5).

### After a DSM update

A DSM update replaces system files and so removes recipes such as the drive database. The app notices in two ways: the background monitor reports a changed DSM version, and the _Extensions_ section compares each recipe's `check` with the journal. A recipe whose last successful `apply` was not undone by a `restore` but whose check now fails is shown as _removed by DSM_, with the options of its last run already chosen.

The optional **boot task** does the same without the app. It is a task in DSM's Task Scheduler (`SYNO.Core.EventScheduler`, event `bootup`, owner root), created over SSH as root with `synowebapi`, so DSM asks for no password confirmation. It runs `~/.syno-companion/boot/reapply.sh`, a script the app writes from the journal and that only root can read or change. For every recipe the journal lists as applied, the script runs `check`, and if that fails, checks the recipe script against its SHA-256 and runs `apply` with the last options, then appends a line by "boot task" to the journal. The app writes the script again after every run while the task is set up.

## Nextcloud

The Nextcloud section (crate `companion-nextcloud`) works over SSH as root, because `occ` must run as the owner of `config/config.php`, usually `http`, which only root can switch to.

- **Finding it:** every `occ` up to two folders below `/volume*/web` is tried with each PHP version of Web Station (`/usr/local/bin/php*`, newest first) until `occ status --output=json` answers. That PHP is the one used for every later command. `config.php` is read with that PHP to learn the database type and whether Redis is configured; passwords are never printed.
- **Compatibility:** `occ update:check` names the next version. Its PHP bounds come from `lib/versioncheck.php` of the matching `stable<major>` branch on GitHub, fetched by the NAS (`PHP_VERSION_ID < 80200` is the minimum, `PHP_VERSION_ID >= 80600` the first version that no longer works). If the PHP in use does not fit, the app names the PHP versions on the NAS that do, so the Web Station profile can be switched first.
- **Guided update:** eight steps, each a separate command whose output is shown, run in order and stopping at the first failure; finished steps are not repeated. 1. `mysqldump --single-transaction` (password via `MYSQL_PWD`, never on a command line) and a copy of `config.php` into `~/.syno-companion/nextcloud/<time>/`; 2. maintenance mode on; 3. files the web server user does not own are given to it, their previous owners noted in `owners.tsv`; 4. `updater/updater.phar --no-interaction`; 5. `occ upgrade`; 6. `occ db:add-missing-indices`, `…-columns`, `…-primary-keys`; 7. the noted owners are restored; 8. maintenance mode off. After a failure, _Leave maintenance mode_ runs steps 7 and 8 alone.
