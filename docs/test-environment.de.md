# Testumgebung

> Deutsche Fassung. Englisches Original: [test-environment.md](test-environment.md)

Syno Companion wird entwickelt und getestet, **ohne ein produktives NAS anzufassen**. Es gibt drei Stufen, von schnell bis realistisch.

## 1. Demomodus (kein NAS)

`npm run dev` startet die Oberfläche im Browser mit Beispieldaten. Gedacht für Layout, Skins und Übersetzungen. Die App selbst zeigt den Demomodus, wenn du auf dem Startbildschirm _Ohne NAS ausprobieren_ wählst.

Mit `/?mock` in der Adresse (`http://localhost:5174/?mock`) läuft im Browser ein Ersatz für den Rust-Kern, sodass sich der ganze Anmeldeablauf durchspielen lässt: Er findet ein NAS, zeigt ein unbekanntes Zertifikat, nimmt das Passwort `demo` und jeden 6-stelligen Code an. Er gehört nur zu `npm run dev`, nie zu einem Release.

## 2. Nachgebauter DSM-Server (die echte App ohne NAS)

`mock-dsm` beantwortet die DSM-Web-API wie ein NAS: mit den von einem Virtual DSM 7.4.1 aufgezeichneten Antworten (ohne persönliche Daten) und Beispieldaten für das, was einem virtuellen NAS fehlt, etwa Laufwerke, Temperatur und Updates. Er liefert HTTPS mit einem frischen selbstsignierten Zertifikat, sodass die Zertifikatsprüfung der App wie bei einem echten DSM läuft.

```bash
cargo run -p companion-mockdsm -- --otp --unconfigured
```

Er zeigt seine Adresse (standardmäßig `https://127.0.0.1:15001`, `--port` ändert sie) und den Fingerabdruck des Zertifikats. Die Adresse in der App eingeben und mit `demo` / `demo` anmelden; mit `--otp` braucht dieses Konto den Code `123456`. Mit `--unconfigured` verhält er sich wie ein NAS, dessen Willkommensassistent noch nicht gelaufen ist, sodass sich die Ersteinrichtung ausprobieren lässt; das dort angelegte Konto meldet sich danach an. Neustart, Herunterfahren und Piepton antworten nur mit „erledigt“.

Die automatischen Tests des DSM-Clients nutzen dieselben aufgezeichneten Antworten. Neue Antworten werden an einem virtuellen DSM aufgezeichnet, nie an einem produktiven NAS.

## 3. Virtuelles DSM (realistische Tests)

Alles, was ein NAS verändert (SSH-Einrichtung, Rezepte, Web Station, Nextcloud), wird zuerst auf einem virtuellen DSM getestet. Synologys Lizenz erlaubt DSM nur auf Synology-Hardware, deshalb laufen beide Varianten auf einem Synology-NAS:

**Variante A – Virtual DSM im Virtual Machine Manager (empfohlen).** Synologys eigener Weg: _Virtual Machine Manager_ aus dem Paket-Zentrum installieren, eine _Virtual DSM_-Instanz anlegen (eine Lizenz ist kostenlos). Braucht ein Modell mit VMM-Unterstützung (die meisten Plus-Modelle mit x86-CPU und mindestens 4 GB RAM). Das virtuelle DSM hat ein eigenes Volume, die Produktivdaten bleiben also außer Reichweite. Vor riskanten Tests einen VMM-Snapshot der Instanz anlegen, danach zurückrollen.

**Variante B – `vdsm/virtual-dsm` im Container Manager.** Der Container [virtual-dsm](https://github.com/vdsm/virtual-dsm) betreibt DSM unter QEMU/KVM. Laut seiner eigenen Dokumentation nur auf Synology-Hardware betreiben. Die DSM-Version wählt man über die Variable `URL` (Link auf die `.pat`-Datei der gewünschten Version). Braucht KVM im Container (`/dev/kvm`).

### Was ein virtuelles DSM nicht testen kann

| Funktion                                | Warum                                             | Wie wir sie stattdessen testen                                                                       |
| --------------------------------------- | ------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Datenbank für Drittanbieter-Festplatten | Virtuelle Platten, keine echten Laufwerksmodelle  | Prüfen/Planen/Wiederherstellen an einer Kopie der Datenbankdateien; Abschlusstest auf Ersatzhardware |
| Speichertest                            | Kein Boot-Test des Synology Assistant in einer VM | Protokolltests mit aufgezeichneten Paketen; Abschlusstest auf echter Hardware                        |
| Lüfter, LEDs, Piepton                   | Keine Hardware                                    | Nachgebauter Server                                                                                  |
| Kernel-Arbeit                           | Anderer Kernel und Bootweg                        | Eigenes Projekt, Ersatzhardware mit serieller Konsole                                                |

### Vorgeschlagener Aufbau

1. Ein virtuelles DSM mit derselben DSM-Version wie das produktive NAS anlegen (derzeit 7.4.1).
2. Einen Admin-Benutzer nur zum Testen anlegen und für ihn 2FA einschalten, damit der Anmeldeablauf mit 2FA getestet wird.
3. Web Station, PHP 8.x, MariaDB 10 und Container Manager installieren; für den Nextcloud-Tab Nextcloud so installieren wie auf dem produktiven NAS.
4. Von diesem Stand einen Snapshot anlegen. Jeder Testlauf beginnt damit.
5. In der App das virtuelle DSM über seine Adresse hinzufügen; es wird als Testgerät markiert.
