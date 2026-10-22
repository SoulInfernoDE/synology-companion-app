# Oberfläche von Syno Companion, deutsche Übersetzung.

-app-name = Syno Companion
-dsm = DSM

unofficial = Unabhängiges Community-Projekt, keine Verbindung zu Synology Inc.

## Navigation

nav-label = Bereiche
nav-overview = Übersicht
nav-files = Freigaben
nav-packages = Pakete
nav-ssh = SSH
nav-extensions = Erweiterungen
nav-nextcloud = Nextcloud
nav-settings = Einstellungen
nav-disconnect = NAS wechseln
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
planned-files = Freigaben mit einem Klick im Dateimanager einbinden, unter Linux, macOS und Windows.
planned-packages = Community-Paketquellen wie SynoCommunity hinzufügen, ihre Pakete durchsuchen und installieren.
planned-ssh = SSH mit Schlüssel einrichten, damit die App auf dem NAS arbeiten kann, ohne jedes Mal nach dem Passwort zu fragen.
planned-extensions = Community-Skripte als sichere Rezepte: jeden Befehl vorher sehen, sichern, anwenden, rückgängig machen. Festplatten von Drittanbietern kommen zuerst.
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
