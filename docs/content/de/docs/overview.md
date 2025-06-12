---
weight: 100
title: Übersicht
description: Atomic Cloud ist eine Gameserver-Cloud, die in Rust geschrieben wurde.
icon: circle
date: 2025-05-07T21:40:07+02:00
lastmod: 2025-05-07T21:40:07+02:00
author: HttpRafa
draft: false
toc: true
tags:
  - Starter
categories:
  - ""
---

Willkommen bei den Atomic Cloud Docs. In diesem Guide zeigen wir dir, wie du mit Atomic Cloud schnell und einfach ein eigenes Netzwerk für Minecraft oder andere Spiele aufbauen kannst. Außerdem stellen wir dir alle Features und verfügbaren Module/Plugins von Atomic Cloud vor, damit du dir genau die Funktionen zusammenstellen kannst, die du für dein Netzwerk brauchst.

## Was ist Atomic Cloud
Atomic Cloud ist eine Gameserver-Cloud, die ursprünglich für Minecraft entwickelt wurde. Dank der Modularität der Cloud ist es aber auch einfach, andere Spiele zu integrieren, die von einem Cloudsystem profitieren können. Die Cloud wurde in [Rust](https://www.rust-lang.org/) programmiert, wodurch sie nur sehr wenig RAM (~20 MB) und kaum CPU-Ressourcen benötigt. Wenn du schnell ein funktionierendes Netzwerk aufbauen willst, schau dir unsere [Schnellstart]({{% relref "quickstart" %}})-Seite an, um zu erfahren, wie du dein Wunsch-Netzwerk einrichtest.

### Funktionen

Atomic Cloud bietet viele [Funktionen]({{% relref "/docs/features" %}}) und [Konfigurationsdateien]({{% relref "/docs/configuration" %}}), die in dieser Dokumentation erklärt werden. Hier ist ein kurzer Überblick über die wichtigsten Funktionen der Cloud:

<div class="row flex-xl-wrap pb-4">

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="{{% relref "/docs/features/groups" %}}">
  <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
    <span class="h1 icon-color">
    <i class="material-icons align-middle">diversity_2</i>
    </span>
    <div class="card-body p-0 content">
    <p class="fs-5 fw-semibold card-title mb-1">Gruppen</p>
    <p class="para card-text mb-0">Server können in Gruppen eingeteilt werden, was zum Beispiel Auto-Skalierung ermöglicht.</p>
    </div>
  </div>
  </a>
</div>

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="{{% relref "/docs/features/nodes" %}}">
  <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
    <span class="h1 icon-color">
    <i class="material-icons align-middle">account_tree</i>
    </span>
    <div class="card-body p-0 content">
    <p class="fs-5 fw-semibold card-title mb-1">Nodes</p>
    <p class="para card-text mb-0">Verteile deine Infrastruktur auf verschiedene Server (Nodes), so wie es für dich passt.</p>
    </div>
  </div>
  </a>
</div>

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="{{% relref "/docs/plugins" %}}">
  <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
    <span class="h1 icon-color">
    <i class="material-icons align-middle">extension</i>
    </span>
    <div class="card-body p-0 content">
    <p class="fs-5 fw-semibold card-title mb-1">Plugins</p>
    <p class="para card-text mb-0">Erstelle Plugins für den Controller in allen Sprachen, die WASM unterstützen.</p>
    </div>
  </div>
  </a>
</div>

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="{{% relref "/docs/features/users" %}}">
  <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
    <span class="h1 icon-color">
    <i class="material-icons align-middle">account_circle</i>
    </span>
    <div class="card-body p-0 content">
    <p class="fs-5 fw-semibold card-title mb-1">User</p>
    <p class="para card-text mb-0">Gib jeder Person nur die Rechte, die sie braucht, um Risiken zu vermeiden.</p>
    </div>
  </div>
  </a>
</div>

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="{{% relref "/docs/features/tls" %}}">
  <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
    <span class="h1 icon-color">
    <i class="material-icons align-middle">shield_locked</i>
    </span>
    <div class="card-body p-0 content">
    <p class="fs-5 fw-semibold card-title mb-1">TLS</p>
    <p class="para card-text mb-0">Verschlüssele deine Kommunikation mit der Cloud.</p>
    </div>
  </div>
  </a>
</div>

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="https://github.com/HttpRafa/atomic-cloud/issues">
  <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
    <span class="h1 icon-color">
    <i class="material-icons align-middle">cognition</i>
    </span>
    <div class="card-body p-0 content">
    <p class="fs-5 fw-semibold card-title mb-1">Noch Ideen?</p>
    <p class="para card-text mb-0">Erstelle ein Issue, um deine Funktion vorzuschlagen.</p>
    </div>
  </div>
  </a>
</div>

</div>