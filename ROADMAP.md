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
- [ ] Mock DSM server with recorded responses for automated tests

## v0.2 – Control

- [ ] Tray icon with status and notifications
- [ ] Wake-on-LAN, restart, shut down, silence the beeper
- [ ] Mount shared folders (SMB) in the file manager of each system
- [ ] Available DSM and package updates
- [ ] Several NAS side by side

## v0.3 – SSH

- [ ] SSH assistant: enable SSH through DSM (after asking), create an ed25519 key or use an existing one, install it with the permissions DSM expects (home 755, `.ssh` 700, `authorized_keys` 600), pin the host key
- [ ] `sudo` through the keychain, never on a command line
- [ ] Built-in terminal for users who want one
- [ ] Diagnostics: DSM build, kernel version, kernel features (`io_uring` and others from `/proc/config.gz` or `/proc/kallsyms`), memory modules with type and speed (`dmidecode`; the DSM Web API reports only the total), network chips and drivers (`ethtool -i`)

## v0.4 – Extend (expert mode)

- [ ] Expert mode with clear warnings
- [ ] Recipe engine: manifest with source repository, pinned version, checksum, steps _check / plan / backup / apply / verify / restore_, licence and supported DSM versions ([architecture](docs/architecture.md#recipes))
- [ ] Journal of every change, exportable
- [ ] First recipe: third-party drives (Synology_HDD_db), including restore
- [ ] Package sources: add SynoCommunity and other feeds, browse and install their packages
- [ ] Container images: check for newer images and recreate containers

## v0.5 – After the update

- [ ] Detect a new DSM version and offer to re-apply every recipe it removed
- [ ] Optional boot task on the NAS that re-applies recipes without the app
- [ ] Update the Docker engine of Container Manager (recipe, expert mode)
- [ ] Memory test like Synology Assistant: research the protocol Synology Assistant uses to start the built-in test; fallback `memtester` over SSH (cannot test memory in use by DSM)

## v1.0 – Nextcloud and stable releases

- [ ] Nextcloud tab: detect the installation (Web Station, PHP profile, database, Redis)
- [ ] Compatibility check: PHP, MariaDB and Redis versions offered by the NAS against the requirements of the next Nextcloud release
- [ ] Guided update: snapshot and database dump, maintenance mode, open permissions, updater, `occ upgrade`, `occ db:add-missing-indices`, close permissions, maintenance mode off; every step visible and resumable
- [ ] Signed packages (Windows, macOS notarisation), Flatpak, AUR
- [ ] Download Station

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
