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
fmContentType: default
---

Willkommen bei den Atomic Cloud Docs. In diesem Guide zeigen wir dir, wie du mit Atomic Cloud schnell und einfach ein eigenes Netzwerk für Minecraft oder andere Spiele aufbauen kannst. Außerdem stellen wir dir alle Features und verfügbaren Module von Atomic Cloud vor, damit du dir genau die Funktionen zusammenstellen kannst, die du für dein Netzwerk brauchst.

## Was ist Atomic Cloud
Atomic Cloud ist eine Gameserver-Cloud, die ursprünglich für Minecraft entwickelt wurde. Dank der Modularität der Cloud ist es jedoch sehr einfach, auch andere Spiele zu integrieren, die von einem Cloudsystem profitieren können. Die Cloud wurde in [Rust](https://www.rust-lang.org/) programmiert, wodurch sie nur sehr wenig RAM (~20 MB) und kaum CPU-Ressourcen benötigt. Wenn du schnell ein funktionierendes Netzwerk aufbauen möchtest, schau dir unsere [Schnellstart]({{% relref "quickstart" %}})-Seite an, um zu erfahren, wie du dein Wunsch-Netzwerk einrichtest.

### Funktionen

Atomic Cloud bietet viele [Funktionen]({{% relref "/docs/features" %}}) und [Konfigurationsdateien]({{% relref "/docs/configuration" %}}), die in dieser Dokumentation erklärt werden. Hier ist ein kurzer Überblick über die wichtigsten Funktionen der Cloud:

<div class="row flex-xl-wrap pb-4">

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="../features/syntax-highlighting/">
  <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
      <span class="h1 icon-color">
        <i class="material-icons align-middle">highlight</i>
      </span>
      <div class="card-body p-0 content">
        <p class="fs-5 fw-semibold card-title mb-1">Syntax Highlighting</p>
        <p class="para card-text mb-0">Highlight your code blocks via PrismJS</p>
      </div>
    </div>
  </a>
</div>

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="../guides/landing-page/overview/">
    <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
      <span class="h1 icon-color">
        <i class="material-icons align-middle">flight_land</i>
      </span>
      <div class="card-body p-0 content">
        <p class="fs-5 fw-semibold card-title mb-1">Landing Page</p>
        <p class="para card-text mb-0">Customizable landing page and templates</p>
      </div>
    </div>
  </a>
</div>

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="../features/docsearch/">
    <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
      <span class="h1 icon-color">
        <i class="material-icons align-middle">search</i>
      </span>
      <div class="card-body p-0 content">
        <p class="fs-5 fw-semibold card-title mb-1">DocSearch</p>
        <p class="para card-text mb-0">A powerful Server Side Search plugin</p>
      </div>
    </div>
  </a>
</div>

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="../features/plausible-analytics/">
    <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
      <span class="h1 icon-color">
        <i class="material-icons align-middle">trending_up</i>
      </span>
      <div class="card-body p-0 content">
        <p class="fs-5 fw-semibold card-title mb-1">Plausible Analytics</p>
        <p class="para card-text mb-0">Open source, Privacy-focused web analytics</p>
      </div>
    </div>
  </a>
</div>

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="../shortcodes/">
    <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
      <span class="h1 icon-color">
        <i class="material-icons align-middle">code</i>
      </span>
      <div class="card-body p-0 content">
        <p class="fs-5 fw-semibold card-title mb-1">Shortcodes</p>
        <p class="para card-text mb-0">Custom shortcodes for when you want to do more than Markdown can offer</p>
      </div>
    </div>
  </a>
</div>

<div id="list-item" class="col-md-4 col-12 py-2">
  <a class="text-decoration-none text-reset" href="../features/feedback-widget/">
    <div class="card h-100 features feature-full-bg rounded p-4 position-relative overflow-hidden border-1">
      <span class="h1 icon-color">
        <i class="material-icons align-middle">reviews</i>
      </span>
      <div class="card-body p-0 content">
        <p class="fs-5 fw-semibold card-title mb-1">Feedback Widget</p>
        <p class="para card-text mb-0">Collect feedback from your visitors on your site’s content</p>
      </div>
    </div>
  </a>
</div>

</div>