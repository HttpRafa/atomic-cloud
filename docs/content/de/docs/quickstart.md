---
weight: 300
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
fmContentType: default
---

## Anforderungen

- **Linux (Ubuntu 24.04.2 LTS) oder Windows**

## Cloud installieren

Lade dir die passende Binärdatei von den GitHub [Releases](https://github.com/HttpRafa/atomic-cloud/releases) herunter:

{{< tabs tabTotal="4">}}
{{% tab tabName="Linux" %}}

Wenn du mit `cd` in den Ordner navigiert bist, in dem du die Cloud installieren möchtest, kannst du den Controller mit dem gezeigten Befehl installieren. Bitte beachte, dass du das Programm `curl` installiert haben musst, damit dies reibungslos funktioniert.

```shell
curl -o controller-linux-x86_64 -L https://github.com/HttpRafa/atomic-cloud/releases/latest/download/controller-linux-x86_64
```

Nachdem du die Binärdatei heruntergeladen hast, kannst du diese ausführbar machen, sodass du den Controller der Cloud später starten kannst, ohne einen Fehler von Linux zu bekommen.

```shell
chmod +x controller-linux-x86_64
```

{{% /tab %}}
{{% tab tabName="Windows" %}}

Lade die Datei manuell auf der Seite herunter und verschiebe sie in den gewünschten Ordner, in dem die Cloud installiert werden soll.

{{% /tab %}}
{{< /tabs >}}

### Mithilfe von Docker installieren

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
