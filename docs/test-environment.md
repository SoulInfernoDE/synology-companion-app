# Test environment

> English original. German version: [test-environment.de.md](test-environment.de.md)

Syno Companion is developed and tested **without touching a production NAS**. There are three levels, from fast to realistic.

## 1. Demo mode (no NAS)

`npm run dev` starts the user interface in the browser with sample data. Use it for layout, skins and translations. The app itself shows demo mode when you choose _Try without a NAS_ on the start screen.

With `/?mock` in the address (`http://localhost:5174/?mock`), a stand-in for the Rust core runs in the browser, so the whole sign-in flow can be tried: it finds one NAS, shows an unknown certificate, accepts the password `demo` and any 6-digit code. It is only part of `npm run dev`, never of a release.

## 2. Mock DSM server (automated tests)

A small server answers DSM Web API calls with responses recorded from a real DSM (personal data removed). Unit and integration tests of the DSM client run against it in CI. Recording new responses is done against a virtual DSM, never against a production NAS.

## 3. Virtual DSM (realistic tests)

Everything that changes a NAS (SSH setup, recipes, Web Station, Nextcloud) is tested on a virtual DSM first. Synology's licence allows DSM only on Synology hardware, so both options run on a Synology NAS:

**Option A – Virtual DSM in Virtual Machine Manager (recommended).** Synology's own way: install _Virtual Machine Manager_ from Package Center, create a _Virtual DSM_ instance (one licence is free). Requires a model with VMM support (most Plus models with x86 CPU and at least 4 GB RAM). The virtual DSM has its own volume, so production data stays out of reach. Before risky tests take a VMM snapshot of the instance; afterwards roll back.

**Option B – `vdsm/virtual-dsm` in Container Manager.** The [virtual-dsm](https://github.com/vdsm/virtual-dsm) container runs DSM under QEMU/KVM. Its own documentation says to run it only on Synology hardware. Choose the DSM version with the `URL` variable (link to the `.pat` file of the wanted version). Needs KVM in the container (`/dev/kvm`).

### What a virtual DSM cannot test

| Feature                    | Why                                     | How we test it instead                                                                      |
| -------------------------- | --------------------------------------- | ------------------------------------------------------------------------------------------- |
| Third-party drive database | Virtual disks, no real drive models     | Check/plan/restore steps against a copy of the database files; final test on spare hardware |
| Memory test                | No Synology Assistant boot test in a VM | Protocol tests against recorded packets; final test on real hardware                        |
| Fan, LEDs, beeper          | No hardware                             | Mock server                                                                                 |
| Kernel work                | Different kernel and boot path          | Separate project, spare hardware with serial console                                        |

### Suggested setup

1. Create a virtual DSM with the same DSM version as the production NAS (currently 7.3).
2. Create an admin user only for testing and enable 2FA for it, so the login flow is tested with 2FA.
3. Install Web Station, PHP 8.x, MariaDB 10 and Container Manager; for the Nextcloud tab install Nextcloud as on the production NAS.
4. Take a snapshot of this state. Every test run starts from it.
5. In the app, add the virtual DSM by address; it is marked as a test device.
