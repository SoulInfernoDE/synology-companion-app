# Syno Companion user interface. English is the base locale.

-app-name = Syno Companion
-dsm = DSM

unofficial = Independent community project, not affiliated with Synology Inc.

## Navigation

nav-label = Sections
nav-overview = Overview
nav-files = Shared folders
nav-packages = Packages
nav-ssh = SSH
nav-extensions = Extensions
nav-nextcloud = Nextcloud
nav-settings = Settings
nav-disconnect = Disconnect
nav-switch = Switch NAS
nav-add = Connect another NAS
nav-expert = Expert

## Start screen

welcome-title = Welcome to { -app-name }
welcome-subtitle = Your Synology NAS on the desktop: status, shared folders and the things { -dsm } does not offer.
welcome-saved = Your NAS
welcome-found = On your network
welcome-searching = Searching the local network …
welcome-search-again = Search again
welcome-none-found = No NAS answered. A firewall on this computer can block the search (UDP port 9999 and Bonjour). You can always enter the address.
welcome-sample = Sample
welcome-connect = Connect
welcome-forget = Remove from the list
welcome-forget-confirm = Remove { $name } from the list? The saved password is deleted too.
welcome-manual = Enter an address
welcome-manual-placeholder = IP address or host name, e.g. 192.168.1.20 or nas.local:5001
welcome-manual-invalid = This is not a valid address.
welcome-demo = Try without a NAS
welcome-back = Back
welcome-demo-hint = Shows sample data. Nothing is sent anywhere.
welcome-browser-only = This is the browser preview. Start the app to connect to a NAS; here you can try the sample data.

## Overview

overview-demo = Sample data. Connect a NAS to see your own.
overview-uptime = Running for { $duration }
overview-dsm = { -dsm } { $version }
overview-kernel = Kernel { $version }
overview-cpu = Processor
overview-cpu-cores =
    { $count ->
        [one] one core
       *[other] { $count } cores
    }
overview-cpu-clock = { NUMBER($ghz, maximumFractionDigits: 1) } GHz
overview-speed-gbit = { NUMBER($value, maximumFractionDigits: 1) } Gbit/s
overview-speed-mbit = { $value } Mbit/s
overview-port-max = up to { $speed }
overview-port-connected = connected
overview-port-disconnected = not connected
overview-memory = Memory
overview-memory-of = of { $total }
overview-network = Network
overview-temperature = System temperature
overview-storage = Storage
overview-volume-used = { $used } of { $total } used
overview-drives = Drives
overview-drive-unlisted = Not on the compatibility list
overview-drive-unlisted-hint = { -dsm } warns about drives that are not on Synology's list. Expert mode will be able to add them.
overview-updates = Updates
overview-update-dsm = { -dsm } { $version } is available
overview-update-dsm-none = { -dsm } is up to date
overview-update-packages =
    { $count ->
        [0] All packages are up to date
        [one] One package can be updated
       *[other] { $count } packages can be updated
    }

health-normal = Healthy
health-warning = Warning
health-critical = Critical

drive-hdd = HDD
drive-ssd = SSD
drive-nvme = NVMe SSD

## Planned sections

planned-version = Planned for version { $version }
planned-extensions = Community scripts as safe recipes: see every command first, back up, apply, undo. Third-party drives come first.
planned-nextcloud = Update Nextcloud in one guided run: check PHP, MariaDB and Redis, take a snapshot, run the updater, close the permissions again.

## Expert mode

expert-locked = This area changes the NAS beyond what { -dsm } offers. Turn on expert mode to use it.
expert-enable = Turn on expert mode
expert-dialog-title = Turn on expert mode?
expert-dialog-body = Expert mode unlocks changes that { -dsm } does not offer. They can affect the stability of your NAS and support from Synology. Every change shows its commands first, makes a backup and can be undone.
expert-dialog-test = Try new changes on a test NAS (Virtual { -dsm }) before your own.
expert-dialog-confirm = Turn on
expert-dialog-cancel = Cancel

## Settings

settings-title = Settings
settings-appearance = Appearance
settings-skin = Skin
settings-skin-auto = Like the system ({ $skin })
settings-scheme = Colors
settings-scheme-auto = Like the system
settings-scheme-light = Light
settings-scheme-dark = Dark
settings-language = Language
settings-language-auto = Like the system
settings-advanced = Advanced
settings-expert = Expert mode
settings-expert-hint = Shows areas that change the NAS beyond { -dsm }'s own settings.
settings-about = About
settings-version = Version { $version }
settings-license = Free software under the GNU GPL 3.0 or later.

volume-name = Volume { $number }

## Sign-in

signin-title = Sign in to { $name }
signin-address = { $host }, port { $port }
signin-account = Account
signin-password = Password
signin-remember-login = Remember the password on this computer
signin-remember-login-hint = Kept in the system keychain, never in a file.
signin-submit = Sign in
signin-cancel = Back
signin-busy = Connecting …
signin-otp-title = Two-step verification
signin-otp-text = Enter the 6-digit code from your authenticator app.
signin-otp-code = Code
signin-remember-device = Remember this computer, so DSM does not ask for a code next time
signin-otp-submit = Confirm
signin-cert-title = Check the certificate
signin-cert-text = The NAS uses a certificate that this computer does not know. This is normal for DSM's own self-signed certificate. Before you trust it, compare the fingerprint with the one your browser shows for the DSM page (padlock › certificate).
signin-cert-changed-title = The certificate has changed
signin-cert-changed-text = The NAS now presents a different certificate than the one you trusted. If you did not replace the certificate in DSM, someone may be intercepting the connection. Only continue if you know why it changed.
signin-cert-fingerprint = SHA-256 fingerprint
signin-cert-trust = Trust this certificate
signin-cert-trust-changed = Trust the new certificate

## Live data

overview-loading = Loading data from the NAS …
overview-refresh-failed = Could not update the data.
overview-account-hint = Some values need an account in the administrators group.

## Errors

error-network = The NAS cannot be reached. Check the address and the port (DSM uses 5001 for HTTPS).
error-wrong-credentials = Account or password is wrong.
error-account-disabled = This account is disabled in DSM.
error-account-not-allowed = This account may not sign in to DSM.
error-otp-wrong = The code is wrong or expired. Try the current one.
error-otp-enforced = DSM requires two-step verification for this account. Set it up in DSM first (Personal › Security).
error-ip-blocked = DSM has blocked this computer's address after too many failed sign-ins. Wait or unblock it in DSM (Control Panel › Security › Protection).
error-password-expired = The password of this account has expired. Change it in DSM first.
error-sign-in-failed = DSM refused the sign-in.
error-session-expired = The session has ended. Please sign in again.
error-permission-denied = This account is not allowed to read this information.
error-not-connected = Not connected to a NAS.
error-discovery = The local network could not be searched.
error-storage = The list of NAS could not be saved.
error-keychain = The system keychain could not be used.
error-unexpected = Something unexpected happened.

## First-time setup

signin-checking = Checking the NAS …
setup-title = Set up { $name }
setup-open-warning = This NAS is not set up yet. Until it is, anyone on your network can manage it without a password. Set it up now.
setup-intro = { -app-name } does what DSM's welcome wizard does: it creates your administrator account, locks the built-in "admin" account and sets the basics. Nothing changes before you confirm.
setup-start = Start setup
setup-device-name = Device name
setup-device-name-hint = How the NAS appears on your network, up to 15 characters.
setup-account-title = Your administrator account
setup-account-hint = Use a name of your own; the built-in "admin" account is disabled.
setup-account-reserved = DSM does not allow this name.
setup-password-repeat = Repeat password
setup-rule-length =
    { $count ->
        [one] At least one character
       *[other] At least { $count } characters
    }
setup-rule-case = Upper and lower case letters
setup-rule-number = At least one number
setup-rule-special = At least one special character
setup-rule-name = Does not contain the account name
setup-rule-repeat = Both passwords match
setup-time-zone = Time zone
setup-updates = DSM updates
setup-updates-important = Install important and security updates automatically (recommended)
setup-updates-all = Install all updates automatically
setup-updates-notifyOnly = Only notify me
setup-review = Review
setup-confirm-title = This will happen on the NAS
setup-do-name = Name the NAS “{ $name }”
setup-do-account = Create the administrator account “{ $account }”
setup-do-admin = Give the built-in “admin” account a random password and disable it
setup-do-zone = Set the time zone to { $zone }
setup-do-updates-important = Install important and security updates automatically
setup-do-updates-all = Install all updates automatically
setup-do-updates-notifyOnly = Only notify about updates
setup-do-privacy = Turn off usage statistics and registration with find.synology.com
setup-do-autoblock = Block addresses after 10 failed sign-ins within 5 minutes
setup-confirm-hint = Your account is created first. The built-in account is only locked afterwards, and only if every step before it worked.
setup-run = Set up NAS
setup-running = Setting up the NAS. This takes a moment …
setup-done = The NAS is set up
setup-skipped = DSM did not accept some of the optional settings. You can change them later in DSM:
setup-continue = Continue
error-setup-expired = The setup form was open too long. Please start the setup again.

## Updates

overview-updates-checking = Checking for updates …
overview-updates-failed = Updates could not be checked.
overview-update-dsm-unknown = DSM could not check for a new version; the NAS may have no internet access.
overview-update-reboot = The NAS restarts during this update.
overview-update-packages-unknown = Package updates could not be checked.
overview-update-install-hint = Install updates in DSM (Control Panel › Update & Restore, Package Center).

## Power

power-beep = Silence the beeper
power-beep-stopped = The beeper is silent.
power-reboot = Restart
power-shutdown = Shut down
power-reboot-title = Restart the NAS?
power-reboot-text = Everyone using the NAS loses the connection for a few minutes. Running copies and backups stop.
power-shutdown-title = Shut down the NAS?
power-shutdown-text = The NAS switches off. It stays off until someone presses its power button or wakes it over the network (Wake-on-LAN).
error-rebooting = The NAS is restarting. Connect again in a few minutes.
error-shutting-down = The NAS is shutting down.

## Shared folders

shares-intro = Open a shared folder in your file manager. You sign in there with your DSM account.
shares-smb-off = The SMB service is off on this NAS. Turn it on in DSM (Control Panel › File Services › SMB) to open shared folders.
shares-none = This account sees no shared folders.
shares-encrypted = Encrypted
shares-open = Open
shares-hint = Linux and macOS mount the folder through smb://, Windows opens it in Explorer.
shares-demo = Sample data. Connect a NAS to open its shared folders.
error-open-share = The shared folder could not be opened.

## Wake-on-LAN

welcome-wake = Wake up (Wake-on-LAN)
welcome-wake-sent = Wake-up sent. If the NAS stays off, turn on Wake-on-LAN in DSM (Control Panel › Hardware & Power).
error-wake-no-mac = The MAC address of this NAS is not known yet; search the network once while it is on.
error-wake = The wake-up could not be sent.

## Tray and notifications

tray-open = Open Syno Companion
tray-quit = Quit
tray-not-connected = Not connected to a NAS
tray-status = { $name }: { $state }
tray-unreachable = not reachable
notify-problem = { $part }: { $state }
notify-healthy = Everything is healthy again.
notify-unreachable = The NAS does not answer.
notify-reachable = The NAS answers again.
notify-dsm-update = DSM { $version } is available.
settings-background = In the background
settings-close-to-tray = Keep running in the tray when the window is closed
settings-close-to-tray-hint = Syno Companion keeps watching the NAS. Open it again from the tray icon.
settings-notifications = Notifications
settings-notifications-hint = About drive and volume problems, a NAS that stops answering and new DSM versions.
error-setup-sign-in = The NAS is set up. Sign in with your new account.

## SSH

ssh-intro = SSH lets Syno Companion look deeper into the NAS and, later, change what DSM does not offer. It signs in with a key of this computer, not with your password.
ssh-setup-title = Set up SSH with a key
ssh-setup-text = Syno Companion needs your DSM password once, to install the key. Only administrators can use SSH on DSM.
ssh-setup-step-enable = Turn on the SSH service in DSM
ssh-setup-step-key = Create a key for this computer
ssh-setup-step-install = Add it to the authorized keys of your account on the NAS
ssh-setup-step-keychain = Keep the key in the system keychain
ssh-port = SSH port
ssh-password = Password of { $account }
ssh-password-hint = Used once to install the key; it is not sent anywhere else.
ssh-setup-link = Set up
ssh-setup-enable-and-link = Turn on SSH and set up
ssh-homes-off = The user home service is off. Keys live in the home folder, so turn it on in DSM first (Control Panel › User & Group › Advanced).
ssh-hostkey-title = Trust this SSH server?
ssh-hostkey-text = This is the first SSH connection to this NAS. Compare the fingerprint with the one in DSM if you want to be sure; Syno Companion remembers it and warns you if it ever changes.
ssh-hostkey-changed-title = The SSH host key has changed
ssh-hostkey-changed-text = The NAS presents a different SSH key than before. That happens after a reset or reinstall of DSM, but it can also mean someone is in between. Only continue if you know why.
ssh-hostkey-trust = Trust and continue
ssh-linked-title = SSH is set up
ssh-linked-text = Signs in as { $account } on port { $port } with the key of this computer.
ssh-unlink = Remove key
ssh-unlink-text = Removes the key from the NAS and from this computer. The SSH service in DSM stays as it is.
ssh-unlink-confirm = Remove
ssh-diag-title = Diagnostics
ssh-diag-refresh = Read again
ssh-diag-dsm = DSM
ssh-diag-update = Update { $update }
ssh-diag-kernel = Kernel
ssh-diag-iouring-available = available
ssh-diag-iouring-disabled = switched off
ssh-diag-iouring-missing = not available
ssh-diag-iouring-hint = io_uring needs Linux 5.1 or newer. Synology's kernel is older on many models.
ssh-diag-network = Network drivers
ssh-diag-no-link = no link
ssh-diag-docker-none = not installed
ssh-diag-memory = Memory modules
ssh-diag-memory-read = Read
ssh-diag-memory-hint = Needs administrator rights (sudo).
ssh-diag-memory-password = Password for sudo:
ssh-diag-unknown = unknown
ssh-terminal-title = Terminal
ssh-terminal-text = A shell on the NAS, signed in with the key.
ssh-terminal-open = Open terminal
ssh-terminal-close = Close
ssh-terminal-hint = Commands run directly on the NAS.
ssh-terminal-ended = The session has ended.
ssh-demo = Sample data: the demo has no NAS to connect to over SSH.
error-ssh-network = The SSH service of the NAS cannot be reached. Check that SSH is on in DSM and the port is right.
error-ssh-host-unknown = The SSH host key is not trusted yet.
error-ssh-host-changed = The SSH host key has changed. Set up SSH again if you know why.
error-ssh-auth = DSM refused the SSH sign-in. Check the password; only administrators can use SSH.
error-ssh-key-rejected = The NAS does not accept the key. Check that the user home service is on, then set up SSH again.
error-ssh-not-linked = SSH is not set up for this NAS yet.
error-ssh-no-home = Your account has no home folder on the NAS. Turn on the user home service in DSM (Control Panel › User & Group › Advanced).
error-ssh-failed = The SSH command failed.
error-password-required = The password is needed for this.
error-sudo = sudo refused the password.

## Packages

packages-intro = Install and update packages, from Synology and from package sources such as SynoCommunity. Installing runs as root over SSH.
packages-installed = Installed
packages-catalogue = Catalogue
packages-sources = Sources
packages-needs-ssh = Installing and updating needs SSH with a key.
packages-set-up-ssh = Set up SSH
packages-password-title = Password for sudo to install { $name }
packages-password-text = Installing runs as root. The password goes to sudo on the NAS and nowhere else.
packages-install = Install
packages-installing = Installing…
packages-update = Update
packages-updating = Updating…
packages-update-all = Update all ({ $count })
packages-done = Done: { $names }
packages-stopped = stopped
packages-needs = needs { $names }
packages-none = No packages installed.
packages-search = Search packages
packages-no-match = No package matches.
packages-wizard-hint = Packages that ask questions while installing (a shared folder, a port) take their defaults; change them in DSM afterwards or install them in Package Center.
packages-sources-warning = Packages of other sources are not checked by Synology and run with the rights of a package on your NAS. Only add sources you trust.
packages-sources-none = No package sources added.
packages-synocommunity-text = Free community packages such as SynoCli, Jellyfin, Transmission and many tools. Maintained by volunteers on GitHub.
packages-source-add = Add
packages-source-remove = Remove
packages-source-other = Another source
packages-source-name = Name
packages-trust = Allow packages of trusted publishers
packages-trust-hint = Package Center then accepts packages whose publisher certificate it trusts, which it does for every added source. Needed for SynoCommunity.
packages-demo = Sample data: nothing is installed in the demo.
error-feed-invalid = DSM cannot read a package list at this address.
error-package-unavailable = A package is not offered for this NAS.
error-package-checksum = The download does not match its checksum and was not installed.
error-package-install = The package could not be installed.
