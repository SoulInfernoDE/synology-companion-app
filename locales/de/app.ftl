# Oberfläche von Syno Companion, deutsche Übersetzung.

-app-name = Syno Companion
-dsm = DSM

unofficial = Unabhängiges Community-Projekt, keine Verbindung zu Synology Inc.

## Navigation

nav-label = Bereiche
nav-overview = Übersicht
nav-files = Freigaben
nav-packages = Pakete
nav-containers = Container
nav-ssh = SSH
nav-extensions = Erweiterungen
nav-nextcloud = Nextcloud
nav-settings = Einstellungen
nav-disconnect = Verbindung trennen
nav-switch = NAS wechseln
nav-add = Weiteres NAS verbinden
nav-expert = Experte

## Startbildschirm

welcome-title = Willkommen bei { -app-name }
welcome-subtitle = Dein Synology-NAS auf dem Desktop: Status, Freigaben und alles, was { -dsm } nicht anbietet.
welcome-saved = Deine NAS
welcome-found = In deinem Netzwerk
welcome-searching = Lokales Netzwerk wird durchsucht …
welcome-search-again = Erneut suchen
welcome-none-found = Kein NAS hat geantwortet. Eine Firewall auf diesem Computer kann die Suche blockieren (UDP-Port 9999 und Bonjour). Die Adresse kannst du immer eingeben.
welcome-sample = Beispiel
welcome-connect = Verbinden
welcome-forget = Aus der Liste entfernen
welcome-forget-confirm = { $name } aus der Liste entfernen? Das gespeicherte Passwort wird ebenfalls gelöscht.
welcome-manual = Adresse eingeben
welcome-manual-placeholder = IP-Adresse oder Hostname, z. B. 192.168.1.20 oder nas.local:5001
welcome-manual-invalid = Das ist keine gültige Adresse.
welcome-demo = Ohne NAS ausprobieren
welcome-back = Zurück
welcome-demo-hint = Zeigt Beispieldaten. Es wird nichts verschickt.
welcome-browser-only = Das ist die Browser-Vorschau. Starte die App, um dich mit einem NAS zu verbinden; hier kannst du die Beispieldaten ausprobieren.

## Übersicht

overview-demo = Beispieldaten. Verbinde ein NAS, um deine eigenen zu sehen.
overview-uptime = Läuft seit { $duration }
overview-dsm = { -dsm } { $version }
overview-kernel = Kernel { $version }
overview-cpu = Prozessor
overview-cpu-cores =
    { $count ->
        [one] ein Kern
       *[other] { $count } Kerne
    }
overview-cpu-clock = { NUMBER($ghz, maximumFractionDigits: 1) } GHz
overview-speed-gbit = { NUMBER($value, maximumFractionDigits: 1) } Gbit/s
overview-speed-mbit = { $value } Mbit/s
overview-port-max = bis { $speed }
overview-port-connected = verbunden
overview-port-disconnected = nicht verbunden
overview-memory = Arbeitsspeicher
overview-memory-of = von { $total }
overview-network = Netzwerk
overview-temperature = Systemtemperatur
overview-storage = Speicher
overview-volume-used = { $used } von { $total } belegt
overview-drives = Laufwerke
overview-drive-unlisted = Nicht auf der Kompatibilitätsliste
overview-drive-unlisted-hint = { -dsm } warnt bei Laufwerken, die nicht auf Synologys Liste stehen. Der Expertenmodus wird sie hinzufügen können.
overview-updates = Updates
overview-update-dsm = { -dsm } { $version } ist verfügbar
overview-update-dsm-none = { -dsm } ist aktuell
overview-update-packages =
    { $count ->
        [0] Alle Pakete sind aktuell
        [one] Ein Paket kann aktualisiert werden
       *[other] { $count } Pakete können aktualisiert werden
    }

health-normal = Fehlerfrei
health-warning = Warnung
health-critical = Kritisch

drive-hdd = HDD
drive-ssd = SSD
drive-nvme = NVMe-SSD

## Geplante Bereiche

planned-version = Geplant für Version { $version }
planned-nextcloud = Nextcloud in einem geführten Durchlauf aktualisieren: PHP, MariaDB und Redis prüfen, Snapshot anlegen, Updater starten, Rechte wieder schließen.

## Expertenmodus

expert-locked = Dieser Bereich verändert das NAS über das hinaus, was { -dsm } anbietet. Schalte den Expertenmodus ein, um ihn zu nutzen.
expert-enable = Expertenmodus einschalten
expert-dialog-title = Expertenmodus einschalten?
expert-dialog-body = Der Expertenmodus schaltet Änderungen frei, die { -dsm } nicht anbietet. Sie können die Stabilität deines NAS und den Support durch Synology beeinträchtigen. Jede Änderung zeigt vorher ihre Befehle, legt eine Sicherung an und lässt sich rückgängig machen.
expert-dialog-test = Probiere neue Änderungen zuerst auf einem Test-NAS (Virtual { -dsm }) aus, bevor du sie auf deinem eigenen nutzt.
expert-dialog-confirm = Einschalten
expert-dialog-cancel = Abbrechen

## Einstellungen

settings-title = Einstellungen
settings-appearance = Darstellung
settings-skin = Skin
settings-skin-auto = Wie das System ({ $skin })
settings-scheme = Farben
settings-scheme-auto = Wie das System
settings-scheme-light = Hell
settings-scheme-dark = Dunkel
settings-language = Sprache
settings-language-auto = Wie das System
settings-advanced = Erweitert
settings-expert = Expertenmodus
settings-expert-hint = Zeigt Bereiche, die das NAS über die Einstellungen von { -dsm } hinaus verändern.
settings-about = Über
settings-version = Version { $version }
settings-license = Freie Software unter der GNU GPL 3.0 oder neuer.

volume-name = Volume { $number }

## Anmeldung

signin-title = Bei { $name } anmelden
signin-address = { $host }, Port { $port }
signin-account = Konto
signin-password = Passwort
signin-remember-login = Passwort auf diesem Computer merken
signin-remember-login-hint = Liegt im Schlüsselbund des Systems, nie in einer Datei.
signin-submit = Anmelden
signin-cancel = Zurück
signin-busy = Verbindung wird hergestellt …
signin-otp-title = Bestätigung in zwei Schritten
signin-otp-text = Gib den 6-stelligen Code aus deiner Authenticator-App ein.
signin-otp-code = Code
signin-remember-device = Diesen Computer merken, damit DSM beim nächsten Mal nicht nach einem Code fragt
signin-otp-submit = Bestätigen
signin-cert-title = Zertifikat prüfen
signin-cert-text = Das NAS nutzt ein Zertifikat, das dieser Computer nicht kennt. Bei DSMs eigenem, selbstsigniertem Zertifikat ist das normal. Bevor du ihm vertraust, vergleiche den Fingerabdruck mit dem, den dein Browser für die DSM-Seite anzeigt (Schloss-Symbol › Zertifikat).
signin-cert-changed-title = Das Zertifikat hat sich geändert
signin-cert-changed-text = Das NAS zeigt jetzt ein anderes Zertifikat als das, dem du vertraut hast. Wenn du das Zertifikat in DSM nicht ersetzt hast, könnte jemand die Verbindung abfangen. Fahre nur fort, wenn du weißt, warum es sich geändert hat.
signin-cert-fingerprint = SHA-256-Fingerabdruck
signin-cert-trust = Diesem Zertifikat vertrauen
signin-cert-trust-changed = Dem neuen Zertifikat vertrauen

## Live-Daten

overview-loading = Daten werden vom NAS geladen …
overview-refresh-failed = Die Daten konnten nicht aktualisiert werden.
overview-account-hint = Einige Werte brauchen ein Konto in der Gruppe der Administratoren.

## Fehler

error-network = Das NAS ist nicht erreichbar. Prüfe Adresse und Port (DSM nutzt 5001 für HTTPS).
error-wrong-credentials = Konto oder Passwort ist falsch.
error-account-disabled = Dieses Konto ist in DSM deaktiviert.
error-account-not-allowed = Dieses Konto darf sich nicht bei DSM anmelden.
error-otp-wrong = Der Code ist falsch oder abgelaufen. Versuche den aktuellen.
error-otp-enforced = DSM verlangt für dieses Konto die Bestätigung in zwei Schritten. Richte sie zuerst in DSM ein (Persönlich › Sicherheit).
error-ip-blocked = DSM hat die Adresse dieses Computers nach zu vielen Fehlversuchen gesperrt. Warte oder hebe die Sperre in DSM auf (Systemsteuerung › Sicherheit › Schutz).
error-password-expired = Das Passwort dieses Kontos ist abgelaufen. Ändere es zuerst in DSM.
error-sign-in-failed = DSM hat die Anmeldung abgelehnt.
error-session-expired = Die Sitzung ist abgelaufen. Bitte melde dich erneut an.
error-permission-denied = Dieses Konto darf diese Informationen nicht lesen.
error-not-connected = Nicht mit einem NAS verbunden.
error-discovery = Das lokale Netzwerk konnte nicht durchsucht werden.
error-storage = Die Liste der NAS konnte nicht gespeichert werden.
error-keychain = Der Schlüsselbund des Systems konnte nicht genutzt werden.
error-unexpected = Etwas Unerwartetes ist passiert.

## Ersteinrichtung

signin-checking = NAS wird geprüft …
setup-title = { $name } einrichten
setup-open-warning = Dieses NAS ist noch nicht eingerichtet. Bis dahin kann jeder in deinem Netzwerk es ohne Passwort verwalten. Richte es jetzt ein.
setup-intro = { -app-name } erledigt, was der Willkommensassistent von DSM tut: Es legt dein Administratorkonto an, sperrt das eingebaute Konto „admin“ und nimmt die Grundeinstellungen vor. Bevor du bestätigst, ändert sich nichts.
setup-start = Einrichtung starten
setup-device-name = Gerätename
setup-device-name-hint = So erscheint das NAS in deinem Netzwerk, bis zu 15 Zeichen.
setup-account-title = Dein Administratorkonto
setup-account-hint = Wähle einen eigenen Namen; das eingebaute Konto „admin“ wird deaktiviert.
setup-account-reserved = Diesen Namen erlaubt DSM nicht.
setup-password-repeat = Passwort wiederholen
setup-rule-length =
    { $count ->
        [one] Mindestens ein Zeichen
       *[other] Mindestens { $count } Zeichen
    }
setup-rule-case = Groß- und Kleinbuchstaben
setup-rule-number = Mindestens eine Ziffer
setup-rule-special = Mindestens ein Sonderzeichen
setup-rule-name = Enthält nicht den Kontonamen
setup-rule-repeat = Beide Passwörter stimmen überein
setup-time-zone = Zeitzone
setup-updates = DSM-Updates
setup-updates-important = Wichtige und Sicherheitsupdates automatisch installieren (empfohlen)
setup-updates-all = Alle Updates automatisch installieren
setup-updates-notifyOnly = Nur benachrichtigen
setup-review = Prüfen
setup-confirm-title = Das passiert auf dem NAS
setup-do-name = Das NAS „{ $name }“ nennen
setup-do-account = Das Administratorkonto „{ $account }“ anlegen
setup-do-admin = Dem eingebauten Konto „admin“ ein Zufallspasswort geben und es deaktivieren
setup-do-zone = Die Zeitzone auf { $zone } setzen
setup-do-updates-important = Wichtige und Sicherheitsupdates automatisch installieren
setup-do-updates-all = Alle Updates automatisch installieren
setup-do-updates-notifyOnly = Bei Updates nur benachrichtigen
setup-do-privacy = Nutzungsstatistiken und die Registrierung bei find.synology.com ausschalten
setup-do-autoblock = Adressen nach 10 fehlgeschlagenen Anmeldungen innerhalb von 5 Minuten sperren
setup-confirm-hint = Dein Konto wird zuerst angelegt. Das eingebaute Konto wird erst danach gesperrt, und nur wenn alle Schritte davor geklappt haben.
setup-run = NAS einrichten
setup-running = Das NAS wird eingerichtet. Das dauert einen Moment …
setup-done = Das NAS ist eingerichtet
setup-skipped = Einige der optionalen Einstellungen hat DSM nicht übernommen. Du kannst sie später in DSM ändern:
setup-continue = Weiter
error-setup-expired = Das Einrichtungsformular war zu lange offen. Bitte starte die Einrichtung erneut.

## Updates

overview-updates-checking = Updates werden geprüft …
overview-updates-failed = Die Updates konnten nicht geprüft werden.
overview-update-dsm-unknown = DSM konnte nicht nach einer neuen Version suchen; vielleicht hat das NAS keinen Internetzugang.
overview-update-reboot = Bei diesem Update startet das NAS neu.
overview-update-packages-unknown = Paket-Updates konnten nicht geprüft werden.
overview-update-install-hint = Updates installierst du in DSM (Systemsteuerung › Aktualisieren & Wiederherstellen, Paket-Zentrum).

## Ein/Aus

power-beep = Piepton abschalten
power-beep-stopped = Der Piepton ist aus.
power-reboot = Neu starten
power-shutdown = Herunterfahren
power-reboot-title = NAS neu starten?
power-reboot-text = Alle, die das NAS nutzen, verlieren für einige Minuten die Verbindung. Laufende Kopien und Sicherungen brechen ab.
power-shutdown-title = NAS herunterfahren?
power-shutdown-text = Das NAS schaltet sich aus. Es bleibt aus, bis jemand den Einschaltknopf drückt oder es über das Netzwerk weckt (Wake-on-LAN).
error-rebooting = Das NAS startet neu. Verbinde dich in einigen Minuten wieder.
error-shutting-down = Das NAS fährt herunter.

## Freigaben

shares-intro = Öffne einen freigegebenen Ordner in deinem Dateimanager. Dort meldest du dich mit deinem DSM-Konto an.
shares-smb-off = Der SMB-Dienst ist auf diesem NAS aus. Schalte ihn in DSM ein (Systemsteuerung › Dateidienste › SMB), um Freigaben zu öffnen.
shares-none = Dieses Konto sieht keine freigegebenen Ordner.
shares-encrypted = Verschlüsselt
shares-open = Öffnen
shares-hint = Linux und macOS binden den Ordner über smb:// ein, Windows öffnet ihn im Explorer.
shares-demo = Beispieldaten. Verbinde ein NAS, um seine Freigaben zu öffnen.
error-open-share = Die Freigabe konnte nicht geöffnet werden.

## Wake-on-LAN

welcome-wake = Aufwecken (Wake-on-LAN)
welcome-wake-sent = Weckruf gesendet. Bleibt das NAS aus, schalte Wake-on-LAN in DSM ein (Systemsteuerung › Hardware & Energie).
error-wake-no-mac = Die MAC-Adresse dieses NAS ist noch nicht bekannt; durchsuche einmal das Netzwerk, während es an ist.
error-wake = Der Weckruf konnte nicht gesendet werden.

## Tray und Benachrichtigungen

tray-open = Syno Companion öffnen
tray-quit = Beenden
tray-not-connected = Mit keinem NAS verbunden
tray-status = { $name }: { $state }
tray-unreachable = nicht erreichbar
notify-problem = { $part }: { $state }
notify-healthy = Alles ist wieder fehlerfrei.
notify-unreachable = Das NAS antwortet nicht.
notify-reachable = Das NAS antwortet wieder.
notify-dsm-update = DSM { $version } ist verfügbar.
notify-dsm-installed = Das NAS läuft jetzt mit { $version }. Wenn du Rezepte nutzt, prüfe unter Erweiterungen, ob sie erneut angewendet werden müssen.
settings-background = Im Hintergrund
settings-close-to-tray = Beim Schließen des Fensters im Tray weiterlaufen
settings-close-to-tray-hint = Syno Companion behält das NAS weiter im Blick. Über das Tray-Symbol öffnest du es wieder.
settings-notifications = Benachrichtigungen
settings-notifications-hint = Bei Problemen mit Laufwerken und Volumes, wenn ein NAS nicht mehr antwortet und bei neuen DSM-Versionen.
error-setup-sign-in = Das NAS ist eingerichtet. Melde dich mit deinem neuen Konto an.

## SSH

ssh-intro = Über SSH kann Syno Companion tiefer in das NAS schauen und später ändern, was DSM nicht anbietet. Die Anmeldung erfolgt mit einem Schlüssel dieses Computers, nicht mit deinem Passwort.
ssh-setup-title = SSH mit Schlüssel einrichten
ssh-setup-text = Syno Companion braucht dein DSM-Passwort einmal, um den Schlüssel zu installieren. Nur Administratoren können SSH auf DSM nutzen.
ssh-setup-step-enable = SSH-Dienst in DSM einschalten
ssh-setup-step-key = Schlüssel für diesen Computer erzeugen
ssh-setup-step-install = Ihn zu den autorisierten Schlüsseln deines Kontos auf dem NAS hinzufügen
ssh-setup-step-keychain = Schlüssel im Schlüsselbund des Systems ablegen
ssh-port = SSH-Port
ssh-password = Passwort von { $account }
ssh-password-hint = Wird einmal zum Installieren des Schlüssels verwendet und nirgendwohin sonst gesendet.
ssh-setup-link = Einrichten
ssh-setup-enable-and-link = SSH einschalten und einrichten
ssh-homes-off = Der Benutzer-Home-Dienst ist aus. Schlüssel liegen im Home-Ordner, schalte ihn also zuerst in DSM ein (Systemsteuerung › Benutzer und Gruppe › Erweitert).
ssh-hostkey-title = Diesem SSH-Server vertrauen?
ssh-hostkey-text = Das ist die erste SSH-Verbindung zu diesem NAS. Vergleiche den Fingerabdruck mit dem in DSM, wenn du sichergehen willst; Syno Companion merkt ihn sich und warnt, falls er sich ändert.
ssh-hostkey-changed-title = Der SSH-Hostschlüssel hat sich geändert
ssh-hostkey-changed-text = Das NAS zeigt einen anderen SSH-Schlüssel als zuvor. Das passiert nach einem Zurücksetzen oder Neuinstallieren von DSM, kann aber auch bedeuten, dass sich jemand dazwischen geschaltet hat. Fahre nur fort, wenn du den Grund kennst.
ssh-hostkey-trust = Vertrauen und fortfahren
ssh-linked-title = SSH ist eingerichtet
ssh-linked-text = Meldet sich als { $account } auf Port { $port } mit dem Schlüssel dieses Computers an.
ssh-unlink = Schlüssel entfernen
ssh-unlink-text = Entfernt den Schlüssel vom NAS und von diesem Computer. Der SSH-Dienst in DSM bleibt, wie er ist.
ssh-unlink-confirm = Entfernen
ssh-diag-title = Diagnose
ssh-diag-refresh = Neu einlesen
ssh-diag-dsm = DSM
ssh-diag-update = Update { $update }
ssh-diag-kernel = Kernel
ssh-diag-iouring-available = verfügbar
ssh-diag-iouring-disabled = abgeschaltet
ssh-diag-iouring-missing = nicht verfügbar
ssh-diag-iouring-hint = io_uring braucht Linux 5.1 oder neuer. Synologys Kernel ist auf vielen Modellen älter.
ssh-diag-network = Netzwerktreiber
ssh-diag-no-link = keine Verbindung
ssh-diag-docker-none = nicht installiert
ssh-diag-memory = Speichermodule
ssh-diag-memory-read = Auslesen
ssh-diag-memory-hint = Braucht Administratorrechte (sudo).
ssh-diag-memory-password = Passwort für sudo:
ssh-diag-unknown = unbekannt
ssh-terminal-title = Terminal
ssh-terminal-text = Eine Shell auf dem NAS, angemeldet mit dem Schlüssel.
ssh-terminal-open = Terminal öffnen
ssh-terminal-close = Schließen
ssh-terminal-hint = Befehle laufen direkt auf dem NAS.
ssh-terminal-ended = Die Sitzung ist beendet.
ssh-demo = Beispieldaten: Die Demo hat kein NAS, mit dem sie sich über SSH verbinden könnte.
error-ssh-network = Der SSH-Dienst des NAS ist nicht erreichbar. Prüfe, ob SSH in DSM eingeschaltet und der Port richtig ist.
error-ssh-host-unknown = Dem SSH-Hostschlüssel wird noch nicht vertraut.
error-ssh-host-changed = Der SSH-Hostschlüssel hat sich geändert. Richte SSH neu ein, wenn du den Grund kennst.
error-ssh-auth = DSM hat die SSH-Anmeldung abgelehnt. Prüfe das Passwort; nur Administratoren können SSH nutzen.
error-ssh-key-rejected = Das NAS nimmt den Schlüssel nicht an. Prüfe, ob der Benutzer-Home-Dienst eingeschaltet ist, und richte SSH dann neu ein.
error-ssh-not-linked = SSH ist für dieses NAS noch nicht eingerichtet.
error-ssh-no-home = Dein Konto hat keinen Home-Ordner auf dem NAS. Schalte den Benutzer-Home-Dienst in DSM ein (Systemsteuerung › Benutzer und Gruppe › Erweitert).
error-ssh-failed = Der SSH-Befehl ist fehlgeschlagen.
error-password-required = Dafür wird das Passwort gebraucht.
error-sudo = sudo hat das Passwort abgelehnt.

## Pakete

packages-intro = Pakete installieren und aktualisieren, von Synology und aus Paketquellen wie SynoCommunity. Die Installation läuft als root über SSH.
packages-installed = Installiert
packages-catalogue = Katalog
packages-sources = Quellen
packages-needs-ssh = Installieren und Aktualisieren braucht SSH mit Schlüssel.
packages-set-up-ssh = SSH einrichten
packages-password-title = Passwort für sudo, um { $name } zu installieren
packages-password-text = Die Installation läuft als root. Das Passwort geht an sudo auf dem NAS und nirgendwohin sonst.
packages-install = Installieren
packages-installing = Wird installiert…
packages-update = Aktualisieren
packages-updating = Wird aktualisiert…
packages-update-all = Alle aktualisieren ({ $count })
packages-done = Fertig: { $names }
packages-stopped = angehalten
packages-needs = braucht { $names }
packages-none = Keine Pakete installiert.
packages-search = Pakete suchen
packages-no-match = Kein Paket passt.
packages-wizard-hint = Pakete, die bei der Installation Fragen stellen (ein freigegebener Ordner, ein Port), übernehmen die Vorgaben; ändere sie danach in DSM oder installiere sie im Paket-Zentrum.
packages-sources-warning = Pakete anderer Quellen prüft Synology nicht, und sie laufen mit den Rechten eines Pakets auf deinem NAS. Füge nur Quellen hinzu, denen du vertraust.
packages-sources-none = Keine Paketquellen hinzugefügt.
packages-synocommunity-text = Kostenlose Community-Pakete wie SynoCli, Jellyfin, Transmission und viele Werkzeuge. Von Freiwilligen auf GitHub gepflegt.
packages-source-add = Hinzufügen
packages-source-remove = Entfernen
packages-source-other = Andere Quelle
packages-source-name = Name
packages-trust = Pakete vertrauenswürdiger Herausgeber zulassen
packages-trust-hint = Das Paket-Zentrum nimmt dann Pakete an, deren Herausgeber-Zertifikat es vertraut, was es für jede hinzugefügte Quelle tut. Für SynoCommunity nötig.
packages-demo = Beispieldaten: In der Demo wird nichts installiert.
error-feed-invalid = DSM kann unter dieser Adresse keine Paketliste lesen.
error-package-unavailable = Ein Paket wird für dieses NAS nicht angeboten.
error-package-checksum = Der Download passt nicht zu seiner Prüfsumme und wurde nicht installiert.
error-package-install = Das Paket konnte nicht installiert werden.

## Erweiterungen (Rezepte)

extensions-intro = Rezepte führen bewährte Community-Skripte auf dem NAS aus: aus einem festen Release, gegen seine Prüfsumme geprüft, mit Sicherung vorher und einem Weg zurück.
extensions-warning = Sie ändern DSM auf eine Weise, die Synology nicht unterstützt. Lies, was ein Rezept tut, sichere deine Daten und probiere neue Rezepte zuerst auf einem Testsystem aus.
extensions-needs-ssh = Rezepte laufen über SSH mit Schlüssel.
extensions-by = von { $author } · { $license }
extensions-applied = angewendet
extensions-not-applied = nicht angewendet
extensions-unsupported = braucht DSM { $dsm }
extensions-options = Optionen
extensions-update-note = Ein DSM-Update kann das rückgängig machen; danach erneut anwenden.
extensions-plan-title = Was passiert
extensions-plan-download = { $file } { $version } in deinen Home-Ordner auf dem NAS laden und die SHA-256-Prüfsumme prüfen
extensions-plan-backup = Als root sichern:
extensions-plan-apply = Als root ausführen:
extensions-plan-restore = Den Wiederherstellungsschritt des Rezepts als root ausführen
extensions-plan-check = Das Ergebnis prüfen
extensions-plan-journal = Den Lauf ins Journal auf dem NAS schreiben
extensions-apply = Anwenden
extensions-apply-again = Erneut anwenden
extensions-restore = Wiederherstellen
extensions-running = Läuft…
extensions-result-ok = Fertig.
extensions-result-failed = Das Rezept ist nicht durchgelaufen. Sonst wurde nichts geändert; siehe die Ausgabe.
extensions-backup-at = Sicherung:
extensions-output = Ausgabe
extensions-journal = Journal
extensions-journal-empty = Auf diesem NAS ist noch kein Rezept gelaufen.
extensions-export = Exportieren
extensions-exported = Das Journal wurde gespeichert.
extensions-applied-action = angewendet
extensions-restored = wiederhergestellt
extensions-demo = Beispieldaten: Die Demo führt keine Rezepte aus.
recipe-third-party-drives-title = Laufwerke anderer Hersteller
recipe-third-party-drives-text = Trägt die eingebauten HDDs, SSDs und M.2-Laufwerke in DSMs Kompatibilitätsdatenbank ein, damit DSM sie nicht mehr als ungeprüft meldet und Funktionen wie M.2-Volumes erlaubt. Nutzt Synology_HDD_db von 007revad.
recipe-third-party-drives-option-noupdate = DSM daran hindern, die Laufwerksdatenbank durch die eigene zu ersetzen
recipe-third-party-drives-option-wdda = WD Device Analytics abschalten (falsche Warnungen für WD-Laufwerke älter als drei Jahre)
recipe-third-party-drives-option-ram = Die Kompatibilitätsprüfung des Arbeitsspeichers abschalten (für RAM anderer Hersteller)
error-recipe-checksum = Das heruntergeladene Skript passt nicht zu seiner Prüfsumme. Es wurde nichts ausgeführt.
error-recipe-dsm = Dieses Rezept braucht ein neueres DSM.
error-recipe-failed = Das Rezept konnte nicht laufen.

## Container

containers-intro = Die Container von Container Manager und welche davon ein älteres Image nutzen, als ihr Tag jetzt anbietet.
containers-needs-ssh = Container werden über SSH mit Schlüssel gelesen, weil Docker auf DSM root braucht.
containers-check-hint = Die Prüfung lädt das neueste Image jedes Tags herunter; laufende Container laufen weiter.
containers-check = Auf Updates prüfen
containers-checking = Wird geprüft…
containers-password = Passwort für sudo
containers-continue = Weiter
containers-update-available = Update
containers-running = läuft
containers-stopped = angehalten
containers-update = Projekt aktualisieren
containers-updating = Wird aktualisiert…
containers-updated = { $project } läuft jetzt mit den aktuellen Images.
containers-single = Einzelne Container
containers-single-hint = Einzelne Container haben ihre Einstellungen nur in Container Manager. Aktualisiere sie dort (Container › Aktion › Zurücksetzen) oder übernimm sie in ein Projekt.
containers-none = Keine Container auf diesem NAS.
containers-current = Alle Container nutzen das neueste Image ihres Tags.
containers-demo = Beispieldaten: Die Demo ändert keine Container.
error-no-docker = Container Manager ist nicht installiert oder läuft nicht.
error-containers = Docker hat einen Fehler gemeldet.
ssh-diag-memtest-passed = Speichertest bestanden ({ $time })
ssh-diag-memtest-failed = Speichertest mit Fehler { $error } fehlgeschlagen ({ $time })
ssh-diag-memtest-none = Kein Speichertest im Systemprotokoll. Synology Assistant kann einen ausführen (das NAS startet neu und ist stundenlang offline).
extensions-removed = von DSM entfernt
extensions-removed-hint = Laut Journal waren diese Rezepte angewendet, jetzt fehlen sie aber, meist nach einem DSM-Update. Wende sie erneut an; die Optionen des letzten Laufs sind schon gewählt.
extensions-boot-title = Nach DSM-Updates
extensions-boot-text = Eine Startaufgabe im Aufgabenplaner von DSM wendet jedes Rezept erneut an, das ein DSM-Update entfernt hat, mit den Optionen des letzten Laufs, auch wenn diese App nicht offen ist. Sie läuft als root und schreibt ins Journal.
extensions-boot-on = an
extensions-boot-off = aus
extensions-boot-add = Startaufgabe einrichten
extensions-boot-remove = Startaufgabe entfernen
