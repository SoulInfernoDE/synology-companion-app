# Architektur

> Deutsche Fassung. Englisches Original: [architecture.md](architecture.md)

## Überblick

```
┌──────────────── Desktop (Linux, macOS, Windows) ────────────────┐
│  UI: Svelte 5 im System-WebView, Skins, Fluent-Übersetzungen     │
│        │ Tauri-Befehle / Ereignisse                              │
│  Kern (Rust)                                                     │
│   ├─ discovery    mDNS/Bonjour, Synology Assistant, Wake-on-LAN │
│   ├─ dsm          Client für die DSM-Web-API (HTTPS, Sitzung, 2FA)│
│   ├─ ssh          Schlüssel, Host-Schlüssel, sudo über Schlüsselbund │
│   ├─ recipes      Manifest, Plan, Sicherung, Anwenden, Prüfen, Rückgängig │
│   ├─ journal      was wann auf welchem NAS geändert wurde        │
│   └─ secrets      Schlüsselbund des Systems                      │
└──────────────────────────┬───────────────────────────────────────┘
          HTTPS (5001)     │      SSH (22 oder eigener Port)
┌──────────────────────────┴───────────────────────────────────────┐
│  Synology-NAS, DSM 7.3+                                           │
└───────────────────────────────────────────────────────────────────┘
```

**Warum Tauri:** eine Codebasis für alle drei Systeme, natives Rust für Netzwerk, SSH und Schlüsselbund, das System-WebView statt eines mitgelieferten Browsers (kleine Pakete) und derselbe Stack wie [overLay](https://github.com/SoulInfernoDE/overLay).

**Zwei Wege zum NAS.** Alles, was DSM anbietet, läuft über die **Web-API**: Sie braucht kein SSH, funktioniert für jeden Benutzer und übersteht DSM-Updates. **SSH** kommt nur dort zum Einsatz, wo die API endet: Rezepte, Diagnose, Nextclouds `occ`. Die Oberfläche zeigt, welchen Weg eine Aktion nimmt.

## Aufbau des Repositorys

| Pfad                         | Inhalt                                                                |
| ---------------------------- | --------------------------------------------------------------------- |
| `crates/companion-app`       | Tauri-App: Fenster, Tray, Befehle                                     |
| `crates/companion-dsm`       | Client für die DSM-Web-API mit Zertifikats-Pinning                    |
| `crates/companion-discovery` | Findet NAS im lokalen Netz (Bonjour, Synology Assistant)              |
| `crates/companion-mockdsm`   | `mock-dsm`, ein nachgebauter DSM, um die App ohne NAS auszuprobieren  |
| `crates/companion-ssh`       | SSH, Schlüssel, sudo, Diagnose                                        |
| `crates/companion-recipes`   | Rezept-Engine und Protokoll                                           |
| `ui/`                        | Oberfläche (Svelte 5, TypeScript, Vite)                               |
| `locales/`                   | Fluent-Kataloge, von Rust und UI gemeinsam genutzt (`en` Basis, `de`) |
| `recipes/`                   | Rezept-Manifeste                                                      |
| `assets/branding/`           | Icon und Logos                                                        |

## NAS finden

Zwei Suchen laufen gleichzeitig drei Sekunden lang:

- **Bonjour/mDNS:** DSM meldet `_http._tcp` mit `vendor=Synology`, Modell, DSM-Version und HTTPS-Port, wenn Bonjour in DSM eingeschaltet ist.
- **Protokoll des Synology Assistant:** eine Anfrage per Broadcast an UDP-Port 9999, die DSMs `findhostd` mit Hostname, IP-Adresse, Modell, DSM-Version und Ports beantwortet, ob Bonjour an ist oder nicht. Ein Paket besteht aus der Kennung `12 34 56 78 'SYNO'` und Einträgen aus einem Byte ID, einem Byte Länge und dem Wert. Der Aufbau folgt öffentlichen Analysen von `findhostd` und [syno_discover.py](https://github.com/007revad/Syno_Toolbox) von 007revad (MIT). Die Anfrage nennt eine lokal verwaltete Absender-MAC, weil findhostd Anfragen von Synologys eigenen MAC-Präfixen ignoriert; sie ist bei jeder Anfrage neu, und die Anfrage geht dreimal hinaus, weil findhostd einem Absender, dem es gerade geantwortet hat, eine Weile nicht antwortet und UDP Pakete verlieren kann. Die Seriennummer in den Antworten wird nicht gespeichert.

Die Antworten werden über die IP-Adresse zusammengeführt. Unter Linux und Windows muss eine Firewall eventuell UDP-Port 9999 hereinlassen; unter macOS fragt das System einmal nach dem Zugriff auf das lokale Netzwerk.

## Anmeldung

1. Die App fragt DSM, welche API-Versionen es anbietet (`SYNO.API.Info`), und meldet sich dann über `SYNO.API.Auth` an. Jeder Aufruf ist ein HTTPS-Formular-POST, damit weder Passwort noch Sitzungs-ID in einer URL landen.
2. **Zertifikate.** Ein Zertifikat, dem das System vertraut (zum Beispiel Let's Encrypt auf DSM), wird wie üblich angenommen. DSMs eigenes, selbstsigniertes Zertifikat nicht: Die App zeigt seinen SHA-256-Fingerabdruck, und sobald der Benutzer ihm vertraut, wird genau dieses Zertifikat für dieses NAS festgehalten. Ein anderes Zertifikat wird später mit einer Warnung abgelehnt, es sei denn, das System vertraut ihm.
3. **Bestätigung in zwei Schritten.** Verlangt DSM einen Code, fragt die App den Benutzer. Mit _Diesen Computer merken_ stellt DSM ein Geräte-Token aus; die nächste Anmeldung schickt es statt eines Codes.
4. **Schlüsselbund.** Das Geräte-Token und, nur auf Wunsch, das Passwort liegen im Schlüsselbund des Systems unter „Syno Companion“, ein Eintrag je NAS und Konto. Die Liste der NAS (`devices.json` im Konfigurationsordner der App) enthält keine Geheimnisse.
5. Beendet DSM eine untätige Sitzung, meldet sich die App still neu an, wenn der Schlüsselbund das Passwort hat; sonst kehrt sie zum Startbildschirm zurück.

## Ersteinrichtung

Solange der Willkommensassistent von DSM nicht gelaufen ist, gibt DSM jedem, der die Weboberfläche öffnet, eine Administrator-Sitzung. Bevor die App nach einem Passwort fragt, prüft sie das: Sie öffnet die Startseite und fragt `SYNO.Core.QuickStart.Info check_permission`. Ein eingerichtetes NAS vergibt keine Sitzung und antwortet mit „keine Sitzung“ (119). (Das Flag `configured` in `SYNO.Core.Desktop.SessionData` hilft nicht: Es bleibt auch nach der Einrichtung `false`.) Bei einem nicht eingerichteten NAS liest sie Passwortregeln, gesperrte Namen und Zeitzonen, zeigt ihr eigenes Formular und schickt nach der Bestätigung einen `SYNO.Entry.Request`, der beim ersten Fehler anhält, in dieser Reihenfolge:

1. Adressen nach Fehlversuchen sperren, Gerätenamen setzen;
2. das Konto des Benutzers anlegen und zu `administrators` hinzufügen;
3. die Registrierung bei find.synology.com stoppen;
4. erst dann dem eingebauten Konto `admin` ein Zufallspasswort geben und es deaktivieren.

Danach meldet sich die App mit dem neuen Konto an und setzt Updates, Zeitzone, Nutzungsstatistiken (aus) und blendet den Willkommensassistenten von DSM aus. Diese späteren Einstellungen sind optional; was DSM ablehnt, wird aufgelistet, die Einrichtung gilt trotzdem als erledigt.

## Ein NAS verwalten

- **Updates:** Seine nächste Version bietet DSM über `SYNO.Core.Upgrade.Server check` an. Für Pakete gibt es keinen einzelnen Aufruf: Die App vergleicht die installierten Versionen (`SYNO.Core.Package list`) mit dem Katalog des Paket-Zentrums (`SYNO.Core.Package.Server list`), so wie das Paket-Zentrum selbst. Die Prüfung fragt Synologys Server, deshalb läuft sie beim Öffnen und danach alle halbe Stunde.
- **Pakete:** Der Katalog führt die installierten Pakete (`SYNO.Core.Package list`) mit dem Angebot des Paket-Zentrums zusammen (`SYNO.Core.Package.Server list` mit `blloadothers`, damit hinzugefügte Quellen wie SynoCommunity dabei sind). Paketquellen werden mit `SYNO.Core.Package.Feed` verwaltet; den Parameter `list` erwartet DSM als JSON-Text in einem JSON-String. SynoCommunity braucht die Vertrauensstufe „Synology Inc. und vertrauenswürdige Herausgeber“ (`SYNO.Core.Package.Setting trust_level=1`), die die App nur setzt, wenn der Haken gesetzt bleibt. Installieren und Aktualisieren laufen über SSH als root: Synologys Pakete mit `synopkg install_from_server`, andere werden auf dem NAS heruntergeladen, gegen die MD5-Prüfsumme der Quelle geprüft und mit `synopkg install` installiert. Fehlende Abhängigkeiten kommen zuerst. Pakete mit Installationsassistent übernehmen dessen Vorgaben.
- **Container** (über SSH als root, da Docker auf DSM root braucht): Die App listet jeden Container mit Image-Tag, Zustand und Compose-Projekt (Labels `com.docker.compose.project` und `…working_dir`). _Auf Updates prüfen_ führt `docker pull` für jeden Tag aus und markiert Container, deren Image nicht mehr das ist, auf das der Tag zeigt. Ein Projekt wird mit `docker compose pull` und `docker compose up -d` in seinem Ordner aktualisiert, sodass Volumes und Einstellungen bleiben, wie das Projekt sie festlegt. Einzelne Container haben ihre Einstellungen nur in Container Manager; die App verweist auf dessen Aktion _Zurücksetzen_, statt die Einstellungen zu erraten.
- **Ein/Aus:** Neustart und Herunterfahren nutzen `SYNO.Core.System reboot`/`shutdown` ohne `force`, damit DSM ablehnt, solange eine Aufgabe nicht unterbrochen werden darf; den Piepton schaltet `SYNO.Core.Hardware.BeepControl stop_beep` ab.
- **Wake-on-LAN:** Die Web-API von DSM meldet keine MAC-Adresse, Bonjour und findhostd aber schon. Die App merkt sich die MAC eines gespeicherten NAS aus der Suche und schickt das Magic Packet ans ganze Netz und an das letzte /24-Netz des NAS, auf den Ports 9 und 7.
- **Freigaben** kommen aus `SYNO.Core.Share list`; versteckte bleiben versteckt. Sie öffnen sich über SMB im Dateimanager des Systems: `smb://`-URLs unter Linux und macOS, `\\host\freigabe` im Explorer unter Windows. Den Ordner `homes` stellt DSM als `home` bereit, den eigenen Home-Ordner.

## SSH

Der SSH-Client ist [russh](https://github.com/Eugeny/russh) (reines Rust, Apache-2.0); ein `ssh`-Programm auf dem Computer ist nicht nötig.

1. **Dienst.** `SYNO.Core.Terminal get` sagt, ob SSH an ist und auf welchem Port; nach Zustimmung schaltet `set` es ein. Die Anmeldung mit Schlüssel braucht den Benutzer-Home-Dienst von DSM (`SYNO.Core.User.Home`), weil `authorized_keys` im Home-Ordner liegt; der Assistent warnt, wenn er aus ist. Nur Administratoren dürfen auf DSM SSH nutzen.
2. **Host-Schlüssel.** Wie beim HTTPS-Zertifikat: Die erste Verbindung zeigt den SHA-256-Fingerabdruck, der Benutzer vertraut ihm, und er wird in `devices.json` festgehalten. Ein anderer Schlüssel wird später mit Warnung abgelehnt.
3. **Schlüssel.** Die App erzeugt pro NAS und Konto einen eigenen ed25519-Schlüssel und meldet sich einmal mit dem DSM-Passwort an (Passwort oder keyboard-interactive), um den öffentlichen Schlüssel an `~/.ssh/authorized_keys` anzuhängen. Dabei bekommt `.ssh` die Rechte 700, die Datei 600, und der Home-Ordner verliert das Schreibrecht für andere, wie sshds `StrictModes` es verlangt. Danach meldet sie sich zur Probe mit dem Schlüssel an und legt erst dann den privaten Schlüssel im Schlüsselbund ab (Eintrag `ssh:konto@nas`). _Schlüssel entfernen_ löscht die Zeile auf dem NAS und den Eintrag im Schlüsselbund.
4. **sudo.** Befehle, die root brauchen, laufen als `sudo -k -S -p '' -- sh -c '…'`; das Passwort geht über die Standardeingabe und taucht so nie in einer Prozessliste oder Shell-History auf. Es kommt aus dem Schlüsselbund, wenn der Benutzer es die App merken lässt, sonst gibt er es ein.
5. **Diagnose** in einem Durchgang: `/etc.defaults/VERSION`, `uname`, `io_uring_setup` in `/proc/kallsyms` plus `kernel.io_uring_disabled`, Treiber und Geschwindigkeit jeder Netzwerkschnittstelle aus sysfs und `docker --version`. Speichermodule kommen aus `dmidecode -t 17` und brauchen sudo.
6. **Terminal** (Expertenmodus): eine PTY-Shell (`xterm-256color`), deren Ausgabe über einen Tauri-Kanal an [xterm.js](https://xtermjs.org/) (MIT) geht; xterm.js wird erst geladen, wenn ein Terminal aufgeht.

## Tray und Benachrichtigungen

Die App kann bei mehreren NAS zugleich angemeldet sein; eines davon ist das aktuelle und wird angezeigt, die anderen bleiben angemeldet. Eine Hintergrundaufgabe fragt jedes angemeldete NAS jede Minute ab, auch bei verstecktem Fenster, und merkt sich, was sie gesehen hat: Das Tray zeigt „Name: Zustand“, und eine Benachrichtigung meldet, wenn ein Volume oder Laufwerk schlechter wird als zuvor, wenn das NAS nicht mehr oder wieder antwortet, wenn alles wieder fehlerfrei ist und, alle sechs Stunden geprüft, wenn DSM eine neue Version anbietet (einmal je Version). Tray-Menü und Benachrichtigungen nutzen dieselben Fluent-Kataloge wie die Oberfläche, eingebettet ins Programm; die Oberfläche gibt ihre Sprache an den Kern weiter. Ob das Schließen des Fensters die App im Tray weiterlaufen lässt und ob Benachrichtigungen erscheinen, steht in `preferences.json` neben der Liste der NAS. Im Tray weiterlaufen ist standardmäßig aus, weil manche Linux-Desktops keine Tray-Symbole zeigen.

## Skins

Ein Skin ist ein Satz CSS-Variablen plus einige Formregeln (Eckenradius, Schrift, Höhe der Bedienelemente, Fokusdarstellung). Die Oberfläche setzt `data-skin="linux|macos|windows"` und `data-scheme="light|dark"` auf `<html>`. Standardmäßig folgt der Skin dem Betriebssystem, auf dem die App läuft, und das Farbschema der Systemeinstellung; beides lässt sich in den Einstellungen überschreiben. Komponenten nutzen nur die Variablen, nie feste Farben, ein neuer Skin ist also ein einziger CSS-Block.

## Übersetzungen

Alle Texte der Oberfläche kommen aus Fluent-Katalogen in `locales/<locale>/*.ftl`. Englisch ist Basis und Rückfallsprache; fehlt eine deutsche Meldung, erscheint sie auf Englisch, nie als rohe ID. Eigennamen (Syno Companion, Synology, DSM, SSH, HTTPS) dürfen direkt im Code stehen.

## Rezepte

Ein Rezept verpackt ein Community-Skript so, dass es sich sicher ausführen lässt. Rezepte liegen in `recipes/*.toml` und sind in die App eingebaut (Crate `companion-recipes`); sie erscheinen im Expertenmodus im Bereich _Erweiterungen_. Das erste:

```toml
id = "third-party-drives"
source = "https://github.com/007revad/Synology_HDD_db"
author = "007revad"
license = "MIT"
version = "v3.6.139"
download = "https://raw.githubusercontent.com/007revad/Synology_HDD_db/v3.6.139/syno_hdd_db.sh"
file = "syno_hdd_db.sh"
sha256 = "e8a392b9…"
dsm = "7.0"                    # älteste DSM-Version
survives-update = false        # ein DSM-Update kann es rückgängig machen
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

Regeln für jedes Rezept:

1. `check` ändert nichts und läuft ohne root; Exit-Status 0 heißt angewendet. Die App zeigt das als Zustand des Rezepts, zusammen damit, ob die DSM-Version neu genug ist.
2. Bevor etwas läuft, zeigt die App den Plan: welche Datei in welcher Version sie lädt, welche Pfade sie sichert und den genauen Befehl mit den gewählten Optionen.
3. Das Skript wird auf dem NAS nach `~/.syno-companion/recipes/<id>/<version>/` geladen und gegen `sha256` geprüft; bei Abweichung bricht alles ab. Direkt bevor es als root läuft, wird es noch einmal geprüft.
4. Als root: Die `backup`-Pfade werden nach `~/.syno-companion/backups/<id>-<zeit>.tar.gz` gesichert (dem Konto gehörend), dann läuft `apply`; `check` sagt, ob es geklappt hat. `restore` macht `apply` mit dem eigenen Wiederherstellungsschritt des Skripts rückgängig.
5. Jeder Lauf wird an `~/.syno-companion/journal.jsonl` auf dem NAS angehängt (Zeit, Rezept und Version, Aktion, Optionen, Ergebnis, DSM-Version und Build, Sicherung, der Computer, der ihn ausgeführt hat), sodass jeder Computer dasselbe Journal sieht; die App kann es als JSON exportieren.
6. Rezepte mit `survives-update = false` listet der Assistent _Nach dem Update_ auf (v0.5).

### Nach einem DSM-Update

Ein DSM-Update ersetzt Systemdateien und entfernt so Rezepte wie die Laufwerksdatenbank. Die App merkt das auf zwei Wegen: Die Überwachung im Hintergrund meldet eine geänderte DSM-Version, und der Bereich _Erweiterungen_ vergleicht das `check` jedes Rezepts mit dem Journal. Ein Rezept, dessen letztes erfolgreiches `apply` kein `restore` rückgängig gemacht hat, dessen Prüfung jetzt aber fehlschlägt, erscheint als _von DSM entfernt_, mit den Optionen des letzten Laufs schon ausgewählt.

Die optionale **Startaufgabe** tut dasselbe ohne die App. Sie ist eine Aufgabe im Aufgabenplaner von DSM (`SYNO.Core.EventScheduler`, Ereignis `bootup`, Besitzer root), angelegt über SSH als root mit `synowebapi`, sodass DSM keine Passwortbestätigung verlangt. Sie startet `~/.syno-companion/boot/reapply.sh`, ein Skript, das die App aus dem Journal schreibt und das nur root lesen oder ändern kann. Für jedes Rezept, das laut Journal angewendet ist, führt das Skript `check` aus, und wenn das fehlschlägt, prüft es das Rezept-Skript gegen seine SHA-256-Prüfsumme und führt `apply` mit den letzten Optionen aus; danach hängt es eine Zeile von „boot task“ an das Journal an. Solange die Aufgabe eingerichtet ist, schreibt die App das Skript nach jedem Lauf neu.
