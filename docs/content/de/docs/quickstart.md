---
weight: 200
title: Schnellstart
description: Eine Anleitung, um mit Atomic Cloud schnell loszulegen.
icon: rocket_launch
date: 2025-05-08T07:23:17.540Z
lastmod: 2025-05-07T21:40:07+02:00
author: HttpRafa
draft: false
toc: true
tags:
  - Starter
  - Guide
categories:
  - ""
---

## Anforderungen

- **Linux (Ubuntu 24.04.2 LTS) oder Windows**
- **Docker (falls du es nutzen möchtest)**
- **Grundkenntnisse im Betriebssystem, das du nutzt**

## Cloud installieren

Bitte wähle die richtige Installationsmethode aus, die du nutzen möchtest. Wir empfehlen dir, Docker auf Linux zu verwenden, wenn du die Cloud im Produktionsnetzwerk nutzen möchtest.

{{< tabs tabTotal="3">}}
{{% tab tabName="Linux" %}}

Wenn du mit `cd` in den Ordner navigiert bist, in dem du die Cloud installieren möchtest, kannst du den Controller mit dem gezeigten Befehl von der GitHub-[Releases](https://github.com/HttpRafa/atomic-cloud/releases) Seite herunterladen. Bitte beachte, dass du das Programm `curl` installiert haben musst, damit dies reibungslos funktioniert.

```shell
curl -o controller-linux-x86_64 -L https://github.com/HttpRafa/atomic-cloud/releases/latest/download/controller-linux-x86_64
```

Nachdem du die Binärdatei heruntergeladen hast, kannst du sie ausführbar machen, damit du den Controller der Cloud später starten kannst, ohne einen Fehler von Linux zu bekommen.

```shell
chmod +x controller-linux-x86_64
```

{{% alert context="info" text="**Hinweis**: Es ist wichtig, dass du eine moderne Linux-Distribution verwendest, da sonst die GLIBC-Version zu alt sein könnte." /%}}

{{% /tab %}}
{{% tab tabName="Linux (Docker)" %}}

Die Cloud kann auch mithilfe von Docker Compose installiert werden. Dafür platziere die Docker-Compose-Datei dort, wo du die Cloud installieren willst.

Hier ist eine Beispiel-Docker-Compose-Datei, die du als Ausgangspunkt nutzen kannst:

```yaml
services:
  controller:
  image: ghcr.io/httprafa/atomic-cloud:latest
  ports:
    - "8080:8080"
  environment:
    - PTERODACTYL=false # Auf true setzen, wenn das Pelican-Plugin installiert werden soll
    - LOCAL=false # Auf true setzen, wenn das Local-Plugin installiert werden soll
    - CLOUDFLARE=false # Auf true setzen, wenn das Cloudflare-Plugin installiert werden soll
  volumes:
    - ./run/certs:/app/certs
    - ./run/configs:/app/configs
    - ./run/groups:/app/groups
    - ./run/logs:/app/logs
    - ./run/nodes:/app/nodes
    - ./run/plugins:/app/plugins
    - ./run/users:/app/users
    - ./run/data:/app/data
```

{{% /tab %}}
{{% tab tabName="Windows" %}}

Lade die Datei `controller-windows-x86_64.exe` manuell auf der GitHub-[Releases](https://github.com/HttpRafa/atomic-cloud/releases) Webseite herunter und verschiebe sie in den gewünschten Ordner, in dem die Cloud installiert werden soll.

{{% /tab %}}
{{< /tabs >}}

## Controller starten {#controller-start}

{{< tabs tabTotal="3">}}
{{% tab tabName="Linux" %}}

Um sicherzustellen, dass der Controller im Hintergrund läuft, auch wenn du die Konsole/SSH schließt, empfehlen wir dir, das Tool `screen` zu installieren.

Wenn du `screen` installiert hast, kannst du den Controller mit diesem Befehl starten:

```shell
screen -S atomic-cloud ./controller-linux-x86_64
```

{{% /tab %}}
{{% tab tabName="Linux (Docker)" %}}

Um den Controller mithilfe von Docker zu starten, führe diesen Befehl aus:

```shell
docker compose up -d
```

Dieser Befehl startet den Controller als Daemon, sodass er immer im Hintergrund läuft. Wenn du die Logs des Controllers sehen möchtest, um mögliche Probleme zu erkennen, nutze diesen Befehl, um die ID des Containers herauszufinden:

```shell
docker ps
```

Wenn du die ID des Containers hast, kannst du mit diesem Befehl die Logs des Containers (Controllers) anzeigen:

```shell
docker logs <ID>
```

{{% /tab %}}
{{% tab tabName="Windows" %}}

Nachdem du die Binärdatei in den Ordner verschoben hast, in dem du die Cloud installieren möchtest, kannst du die Datei `controller-windows-x86_64.exe` doppelklicken, um den Controller zu starten.

{{% /tab %}}
{{< /tabs >}}

### Beispiel

![Startup](/images/controller/startup.png)

## CLI installieren

Als nächstest müssen wir die CLI für die Cloud installieren. Die CLI emöglicht es uns mit dem Controller zu reden ohne SSH zugriff zu benötigen was bei der Rechte verteilung wichtig ist.

Die CLI losste auf deinem Computer installiert werden von wo du denn Controller kontrollieren möchtest. Sie muss nicht auf dem Server installiert werden wo der Controller denn wir eben installiert habe leuft.

{{< tabs tabTotal="2">}}
{{% tab tabName="Linux" %}}

Die CLI kann ebenfalls von der GitHub-[Releases](https://github.com/HttpRafa/atomic-cloud/releases) Seite herunterladen werden. Bitte beachte, dass du das Programm `curl` installiert haben musst, damit dies reibungslos funktioniert.

```shell
curl -o cli-linux-x86_64 -L https://github.com/HttpRafa/atomic-cloud/releases/latest/download/cli-linux-x86_64
```

Nachdem du die Binärdatei heruntergeladen hast, kannst du sie ausführbar machen, damit du die CLI der Cloud später starten kannst, ohne einen Fehler von Linux zu bekommen.

```shell
chmod +x cli-linux-x86_64
```

{{% alert context="info" text="**Hinweis**: Es ist wichtig, dass du eine moderne Linux-Distribution verwendest, da sonst die GLIBC-Version zu alt sein könnte." /%}}

{{% /tab %}}
{{% tab tabName="Windows" %}}

Lade die Datei `cli-windows-x86_64.exe` manuell auf der GitHub-[Releases](https://github.com/HttpRafa/atomic-cloud/releases) Webseite herunter und verschiebe sie in den gewünschten Ordner, in dem die Cloud installiert werden soll.

{{% /tab %}}
{{< /tabs >}}

## CLI Einrichten

Nachdem wir die CLI installiert haben können wir sie starten. Der start vorgang ist sehr iden tische wie von dem Controller. Verfolge die selben anweisungen aber passe die Datei die du ausführst an zu der CLI binär datei. Siehe: [Controller starten](#controller-start)

### Verbindung informationen eintragen