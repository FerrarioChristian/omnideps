# Omnideps Website & Documentation

Official product website and documentation portal for **Omnideps** — a language-agnostic architectural dependency analyzer built in Rust and powered by Tree-sitter.

Built with **SvelteKit 2**, **Svelte 5**, **Tailwind CSS v4**, **shadcn-svelte** (Bits UI), **MDSvex**, and **Pagefind**.

---

## Getting Started

### 1. Install Dependencies
```bash
npm install
```

### 2. Start Local Development Server
```bash
npm run dev
```

Visit [http://localhost:5173](http://localhost:5173) in your browser.

### 3. Production Build & Pagefind Indexing
```bash
npm run build
```

This compiles the static/prerendered site and automatically builds the offline full-text search index (`Pagefind`) used by the `Cmd+K` command palette.

### 4. Preview Production Build
```bash
npm run preview
```

---

## Project Structure

```
website/
├── src/
│   ├── content/
│   │   └── docs/             # Markdown docs with frontmatter (MDSvex)
│   │       ├── getting-started/
│   │       ├── cli/
│   │       ├── architecture/
│   │       └── guides/
│   ├── lib/
│   │   ├── components/       # shadcn-svelte UI components & layouts
│   │   ├── docs/             # docsConfig, navigation & search
│   │   └── assets/           # favicon, SVG icons
│   └── routes/
│       ├── +page.svelte      # Main product landing page (Hero, OS Download, Bento Grid)
│       └── (docs)/docs/      # Documentation layout and markdown renderer
└── svelte.config.js
```
