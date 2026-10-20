import { defineConfig } from "vitepress"

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Atomic Cloud",
  description: "A program that makes it possible to put special game servers in a network",
  themeConfig: {
    logo: "/icon.png",

    nav: [
      { text: "Home", link: "/" },
      { text: "Documentation", link: "/quick-start" }
    ],

    sidebar: [
      {
        text: "Basics",
        items: [
          { text: "Quick Start", link: "/quick-start" }
        ]
      },
      {
        text: "Configuration",
        items: [
          { text: "Introduction", link: "/configuration/introduction" }
        ]
      }
    ],

    socialLinks: [
      { icon: "github", link: "https://github.com/HttpRafa/atomic-cloud" }
    ],

    footer: {
      message: 'Released under the GPL-3.0 License.',
      copyright: 'Copyright © 2025-present HttpRafa'
    },

    editLink: {
      pattern: 'https://github.com/HttpRafa/atomic-cloud/edit/main/docs/:path',
      text: 'Edit this page on GitHub'
    }
  }
})
