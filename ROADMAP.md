# Roadmap

> English original. German version: [ROADMAP.de.md](ROADMAP.de.md)

Each version is usable on its own. Features that change the NAS beyond DSM's own settings land behind expert mode and only after they have been tested on a virtual DSM ([test environment](docs/test-environment.md)).

## v0.1 – See

- [x] App shell: Tauri 2, Svelte 5, Fluent translations (English base, German)
- [x] Skins for Linux, macOS and Windows, light and dark, automatic or chosen by hand
- [x] Demo mode with sample data (UI development without a NAS, screenshots)
- [x] Find NAS on the local network (mDNS/Bonjour)
- [x] Find NAS without Bonjour, the way Synology Assistant does (UDP broadcast)
- [x] Sign in through the DSM Web API, with 2FA and "remember this device"
- [x] Self-signed DSM certificates: show the fingerprint, pin it once trusted, warn when it changes
- [x] Credentials in the system keychain (Secret Service, macOS Keychain, Windows Credential Manager)
- [x] Saved NAS: connect again with one click, remove from the list
- [x] First-time setup of a NAS whose DSM welcome wizard has not run: own administrator account, built-in `admin` locked, device name, time zone, updates; usage statistics off
- [x] First-time setup tested on a Virtual DSM 7.4.1
- [x] Dashboard: system, utilisation, volumes, drives
- [x] Check the dashboard fields against a Virtual DSM 7.4.1 and record its answers as test fixtures (drives and temperature need real hardware)
- [x] CI: checks on Linux, macOS and Windows; licence and advisory checks
- [x] Release builds for Linux x86_64 and arm64 (deb, rpm, AppImage), macOS universal (dmg) and Windows (msi, exe) ([releasing](docs/releasing.md))

## v0.2 – Control

- [x] Tray icon with status and notifications (drive and volume problems, NAS not answering, new DSM version)
- [x] Wake-on-LAN, restart, shut down, silence the beeper
- [x] Open shared folders (SMB) in the file manager of each system
- [x] Available DSM and package updates
- [x] Install package updates from the app (v0.4, over SSH with `synopkg`); DSM updates stay in DSM, where the update can be followed on the screen of the NAS
- [x] Several NAS side by side: switch in the sidebar, disconnect one, tray and notifications watch all
- [x] Standalone mock DSM server from the recorded answers, to try the app without a NAS, with two-step verification and first-time setup

## v0.3 – SSH

- [x] SSH assistant: enable SSH through DSM (after asking), create an ed25519 key for this computer, install it with the permissions sshd expects (`.ssh` 700, `authorized_keys` 600, home not writable by others), pin the host key. The app uses a key of its own rather than an existing one, so it can be removed without touching your other keys.
- [x] `sudo` with the password from the keychain (or typed once), sent on standard input, never on a command line
- [x] Built-in terminal (expert mode)
- [x] Diagnostics: DSM build, kernel version, `io_uring` (from `/proc/kallsyms` and `kernel.io_uring_disabled`), memory modules with type and speed (`dmidecode`; the DSM Web API reports only the total), network drivers and link speed (sysfs)

## v0.4 – Extend (expert mode)

- [x] Expert mode with clear warnings
- [x] Recipe engine: manifest with source repository, pinned version, checksum, steps _check / plan / backup / apply / verify / restore_, licence and supported DSM versions ([architecture](docs/architecture.md#recipes))
- [x] Journal of every change, kept on the NAS, exportable
- [x] First recipe: third-party drives (Synology_HDD_db), including restore
- [x] Package sources: add SynoCommunity and other feeds, browse and install their packages (with the packages they depend on)
- [x] Container images: check for newer images and recreate the containers of Container Manager projects (single containers: a hint to reset them in Container Manager)

## v0.5 – After the update

- [x] Detect a new DSM version (notification) and offer to re-apply every recipe it removed, with the options of the last run
- [x] Optional boot task on the NAS that re-applies recipes without the app
- [ ] Update the Docker engine of Container Manager (recipe, expert mode). Researched: the known script ([markdumay/synology-docker](https://github.com/markdumay/synology-docker), last release 2021) was tested only up to DSM 7.0.1, before Container Manager. Not offered until a maintained script exists.
- [x] Memory test results: `findhostd` logs the result of Synology Assistant's test ("Memtest passed!" or "Memtest failed! Err=…") in `/var/log/messages`; the diagnostics show it next to the memory modules
- [ ] Start the memory test from the app: Synology Assistant asks `findhostd` over UDP 9999 to restart into the test. The command has to be captured from Synology Assistant on real hardware first (Virtual DSM has no memory test). `memtester` over SSH was dropped: it cannot test the memory DSM uses, which is where faults matter.

## v1.0 – Nextcloud and stable releases

- [x] Nextcloud tab: detect the installation (Web Station, PHP version, database, Redis)
- [x] Compatibility check: the PHP version against the bounds of the next Nextcloud release (from its `lib/versioncheck.php`); MariaDB and Redis versions are shown
- [x] Guided update: database dump, maintenance mode, open permissions, updater, `occ upgrade`, `occ db:add-missing-indices`, close permissions, maintenance mode off; every step visible and resumable. The app does not take a snapshot itself; it recommends one in Snapshot Replication first.
- [x] Packaging ready: optional signing in the release workflow (Windows certificate, macOS signing and notarisation), Flatpak manifest, AUR package
- [ ] Publish signed packages, on Flathub and in the AUR (needs certificates and accounts of the maintainer)
- [x] Download Station: list tasks with progress, add links (HTTP, HTTPS, FTP, magnet), pause, resume, remove

## v2.0 – Web Station

- [ ] Web Station profiles: PHP values and modules (for example `redis`) that survive DSM updates
- [ ] Nextcloud presets (memory limit, OPcache, APCu, Redis) as one click
- [ ] Optional: MCP server, so AI assistants can read the NAS status (like Git-Switch)

## Separate project: kernel toolchain

A second repository for a cross-compiler toolchain that backports current kernel.org kernels to Synology models. This is research first, because:

- DSM kernels carry Synology's own patches and drivers (hardware monitoring, LED and fan control, `synobios`, Btrfs and RAID changes). Synology publishes the GPL sources per DSM version; they have to be compared with mainline and ported.
- DSM updates replace the kernel. A safe setup needs a **fallback kernel**: if the new kernel does not boot, the NAS starts the Synology kernel again. Whether and how this works depends on the boot loader of each model family; a serial console is needed for development.
- `io_uring` cannot be loaded as a module; it needs a rebuilt kernel. The v0.3 diagnostics show whether the running kernel already has it.

Until this has been researched and tested on spare hardware, the app will not install kernels.
