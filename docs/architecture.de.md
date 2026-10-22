# Architektur

> Deutsche Fassung. Englisches Original: [architecture.md](architecture.md)

## Überblick

```
┌──────────────── Desktop (Linux, macOS, Windows) ────────────────┐
│  UI: Svelte 5 im System-WebView, Skins, Fluent-Übersetzungen     │
│        │ Tauri-Befehle / Ereignisse                              │
│  Kern (Rust)                                                     │
│   ├─ discovery    mDNS/Bonjour, Wake-on-LAN                      │
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
| `crates/companion-discovery` | Findet NAS im lokalen Netz (mDNS/Bonjour)                             |
| `crates/companion-ssh`       | SSH, Schlüssel, sudo _(geplant)_                                      |
| `crates/companion-recipes`   | Rezept-Engine und Protokoll _(geplant)_                               |
| `ui/`                        | Oberfläche (Svelte 5, TypeScript, Vite)                               |
| `locales/`                   | Fluent-Kataloge, von Rust und UI gemeinsam genutzt (`en` Basis, `de`) |
| `recipes/`                   | Rezept-Manifeste _(geplant)_                                          |
| `assets/branding/`           | Icon und Logos                                                        |

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

## Skins

Ein Skin ist ein Satz CSS-Variablen plus einige Formregeln (Eckenradius, Schrift, Höhe der Bedienelemente, Fokusdarstellung). Die Oberfläche setzt `data-skin="linux|macos|windows"` und `data-scheme="light|dark"` auf `<html>`. Standardmäßig folgt der Skin dem Betriebssystem, auf dem die App läuft, und das Farbschema der Systemeinstellung; beides lässt sich in den Einstellungen überschreiben. Komponenten nutzen nur die Variablen, nie feste Farben, ein neuer Skin ist also ein einziger CSS-Block.

## Übersetzungen

Alle Texte der Oberfläche kommen aus Fluent-Katalogen in `locales/<locale>/*.ftl`. Englisch ist Basis und Rückfallsprache; fehlt eine deutsche Meldung, erscheint sie auf Englisch, nie als rohe ID. Eigennamen (Syno Companion, Synology, DSM, SSH, HTTPS) dürfen direkt im Code stehen.

## Rezepte

Ein Rezept verpackt ein Community-Skript oder eine Reihe von Befehlen so, dass ein normaler Benutzer es sicher ausführen kann. Beispiel (Format nicht endgültig):

```toml
id = "third-party-drives"
name = "recipe-third-party-drives"          # Fluent-Meldungs-ID
source = "https://github.com/007revad/Synology_HDD_db"
version = "v3.6.139"
sha256 = "…"
license = "MIT"
dsm = ">=7.3"
expert = true
survives-update = false                       # nach DSM-Updates neu einspielen

[steps]
check   = "syno_hdd_db.sh --showedits"  # nur lesend
backup  = "builtin:backup /var/lib/disk-compatibility"
apply   = "syno_hdd_db.sh --noupdate"
verify  = "syno_hdd_db.sh --showedits"
restore = "syno_hdd_db.sh --restore"
```

Regeln für jedes Rezept:

1. Das Skript wird aus dem festgelegten Release geladen und gegen `sha256` geprüft; bei Abweichung bricht alles ab.
2. `check` und `plan` ändern nichts; ihre Ausgabe wird angezeigt, bevor der Benutzer bestätigt.
3. `backup` läuft vor `apply`; `restore` muss `apply` vollständig rückgängig machen.
4. Jeder Lauf wird ins Protokoll geschrieben (Zeit, NAS, DSM-Build, Rezeptversion, Ergebnis).
5. Rezepte mit `survives-update = false` listet der Assistent _Nach dem Update_ auf.
