# Roadmap

> Deutsche Fassung. Englisches Original: [ROADMAP.md](ROADMAP.md)

Jede Version ist für sich nutzbar. Funktionen, die das NAS über DSMs eigene Einstellungen hinaus verändern, liegen hinter dem Expertenmodus und kommen erst, wenn sie auf einem virtuellen DSM getestet sind ([Testumgebung](docs/test-environment.de.md)).

## v0.1 – Sehen

- [x] Grundgerüst: Tauri 2, Svelte 5, Fluent-Übersetzungen (Englisch als Basis, Deutsch)
- [x] Skins für Linux, macOS und Windows, hell und dunkel, automatisch oder von Hand gewählt
- [x] Demomodus mit Beispieldaten (UI-Entwicklung ohne NAS, Screenshots)
- [x] NAS im lokalen Netz finden (mDNS/Bonjour)
- [ ] NAS ohne Bonjour finden, so wie Synology Assistant (UDP-Broadcast)
- [x] Anmeldung über die DSM-Web-API, mit 2FA und „Gerät merken“
- [x] Selbstsignierte DSM-Zertifikate: Fingerabdruck zeigen, nach dem Vertrauen festhalten, bei Änderung warnen
- [x] Zugangsdaten im Schlüsselbund (Secret Service, macOS-Schlüsselbund, Windows-Anmeldeinformationsverwaltung)
- [x] Gespeicherte NAS: mit einem Klick wieder verbinden, aus der Liste entfernen
- [x] Ersteinrichtung eines NAS, dessen DSM-Willkommensassistent noch nicht gelaufen ist: eigenes Administratorkonto, eingebautes `admin` gesperrt, Gerätename, Zeitzone, Updates; Nutzungsstatistiken aus
- [ ] Ersteinrichtung am Virtual DSM 7.4.1 ausprobieren (vorher Snapshot, danach zurückrollen)
- [x] Übersicht: System, Auslastung, Volumes, Laufwerke
- [x] Felder der Übersicht an einem Virtual DSM 7.4.1 prüfen und seine Antworten als Testdaten aufzeichnen (Laufwerke und Temperatur brauchen echte Hardware)
- [x] CI: Prüfungen unter Linux, macOS und Windows; Lizenz- und Sicherheitsprüfung
- [x] Release-Builds für Linux x86_64 und arm64 (deb, rpm, AppImage), macOS universell (dmg) und Windows (msi, exe) ([Releases](docs/releasing.de.md))
- [ ] Nachgebauter DSM-Server mit aufgezeichneten Antworten für automatische Tests

## v0.2 – Steuern

- [ ] Tray-Icon mit Status und Benachrichtigungen
- [ ] Wake-on-LAN, Neustart, Herunterfahren, Piepton abschalten
- [ ] Freigaben (SMB) im Dateimanager des jeweiligen Systems einbinden
- [ ] Verfügbare DSM- und Paket-Updates
- [ ] Mehrere NAS nebeneinander

## v0.3 – SSH

- [ ] SSH-Assistent: SSH über DSM einschalten (nach Rückfrage), ed25519-Schlüssel erzeugen oder vorhandenen nehmen, mit den Rechten hinterlegen, die DSM erwartet (Home 755, `.ssh` 700, `authorized_keys` 600), Host-Schlüssel merken
- [ ] `sudo` über den Schlüsselbund, nie in einer Befehlszeile
- [ ] Eingebautes Terminal für alle, die eins möchten
- [ ] Diagnose: DSM-Build, Kernel-Version, Kernel-Funktionen (`io_uring` und andere aus `/proc/config.gz` oder `/proc/kallsyms`)

## v0.4 – Erweitern (Expertenmodus)

- [ ] Expertenmodus mit deutlichen Warnungen
- [ ] Rezept-Engine: Manifest mit Quell-Repository, fester Version, Prüfsumme, Schritten _prüfen / planen / sichern / anwenden / kontrollieren / wiederherstellen_, Lizenz und unterstützten DSM-Versionen ([Architektur](docs/architecture.de.md#rezepte))
- [ ] Protokoll jeder Änderung, exportierbar
- [ ] Erstes Rezept: Festplatten von Drittanbietern (Synology_HDD_db), mit Wiederherstellung
- [ ] Paketquellen: SynoCommunity und andere Quellen hinzufügen, Pakete durchsuchen und installieren
- [ ] Container-Images: auf neuere Images prüfen und Container neu erstellen

## v0.5 – Nach dem Update

- [ ] Neue DSM-Version erkennen und anbieten, alle Rezepte wieder einzuspielen, die sie entfernt hat
- [ ] Optionale Startaufgabe auf dem NAS, die Rezepte auch ohne die App wieder einspielt
- [ ] Docker-Engine von Container Manager aktualisieren (Rezept, Expertenmodus)
- [ ] Speichertest wie im Synology Assistant: das Protokoll erforschen, mit dem Synology Assistant den eingebauten Test startet; Ausweichlösung `memtester` über SSH (kann den von DSM belegten Speicher nicht testen)

## v1.0 – Nextcloud und stabile Releases

- [ ] Nextcloud-Tab: Installation erkennen (Web Station, PHP-Profil, Datenbank, Redis)
- [ ] Kompatibilitätsprüfung: PHP-, MariaDB- und Redis-Versionen des NAS gegen die Anforderungen des nächsten Nextcloud-Releases
- [ ] Geführtes Update: Snapshot und Datenbank-Dump, Wartungsmodus, Rechte öffnen, Updater, `occ upgrade`, `occ db:add-missing-indices`, Rechte schließen, Wartungsmodus aus; jeder Schritt sichtbar und fortsetzbar
- [ ] Signierte Pakete (Windows, macOS-Notarisierung), Flatpak, AUR
- [ ] Download Station

## v2.0 – Web Station

- [ ] Web-Station-Profile: PHP-Werte und -Module (zum Beispiel `redis`), die DSM-Updates überstehen
- [ ] Nextcloud-Voreinstellungen (Speicherlimit, OPcache, APCu, Redis) mit einem Klick
- [ ] Optional: MCP-Server, damit KI-Assistenten den NAS-Status lesen können (wie bei Git-Switch)

## Eigenes Projekt: Kernel-Toolchain

Ein zweites Repository für eine Cross-Compiler-Toolchain, die aktuelle Kernel von kernel.org auf Synology-Modelle zurückportiert. Das ist zuerst Recherche, denn:

- DSM-Kernel enthalten Synologys eigene Patches und Treiber (Hardwareüberwachung, LED- und Lüftersteuerung, `synobios`, Änderungen an Btrfs und RAID). Synology veröffentlicht die GPL-Quellen je DSM-Version; sie müssen mit dem Mainline-Kernel verglichen und portiert werden.
- DSM-Updates ersetzen den Kernel. Ein sicherer Aufbau braucht einen **Ausweich-Kernel**: Bootet der neue Kernel nicht, startet das NAS wieder mit dem Synology-Kernel. Ob und wie das geht, hängt vom Bootloader der jeweiligen Modellfamilie ab; für die Entwicklung braucht es eine serielle Konsole.
- `io_uring` lässt sich nicht als Modul nachladen, dafür muss der Kernel neu gebaut werden. Die Diagnose aus v0.3 zeigt, ob der laufende Kernel es schon enthält.

Bis das erforscht und auf Ersatzhardware getestet ist, installiert die App keine Kernel.
