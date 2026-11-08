---
title: Installation
description: How to install Omnideps via pre-compiled standalone binaries or build from source.
order: 1
---

# Installing Omnideps

Omnideps is distributed as a **single standalone binary**. The web visualizer is pre-compiled and embedded directly inside the Rust executable, meaning **end users do not need to install Node.js, Python, or even Rust** to run it.

---

## 1. Pre-compiled Binaries (Recommended)

Download the pre-compiled binary matching your platform:

<div class="not-prose my-6 grid gap-4 sm:grid-cols-3">
  <div class="rounded-lg border bg-card p-4">
    <div class="text-xs uppercase tracking-wider text-muted-foreground font-semibold">macOS</div>
    <div class="text-sm font-semibold mt-1">Standalone Binary</div>
    <div class="mt-3 flex flex-col gap-1.5 text-xs">
      <a href="https://github.com/FerrarioChristian/omnideps/releases/latest/download/omnideps-macos-aarch64" class="underline hover:text-foreground font-medium">Apple Silicon (arm64) ↓</a>
      <a href="https://github.com/FerrarioChristian/omnideps/releases/latest/download/omnideps-macos-x86_64" class="underline hover:text-foreground font-medium">Intel (x86_64) ↓</a>
    </div>
  </div>
  <div class="rounded-lg border bg-card p-4">
    <div class="text-xs uppercase tracking-wider text-muted-foreground font-semibold">Linux</div>
    <div class="text-sm font-semibold mt-1">Standalone ELF</div>
    <div class="mt-3 flex flex-col gap-1.5 text-xs">
      <a href="https://github.com/FerrarioChristian/omnideps/releases/latest/download/omnideps-linux-x86_64" class="underline hover:text-foreground font-medium">Linux x86_64 Binary ↓</a>
    </div>
  </div>
  <div class="rounded-lg border bg-card p-4">
    <div class="text-xs uppercase tracking-wider text-muted-foreground font-semibold">Windows</div>
    <div class="text-sm font-semibold mt-1">Executable</div>
    <div class="mt-3 flex flex-col gap-1.5 text-xs">
      <a href="https://github.com/FerrarioChristian/omnideps/releases/latest/download/omnideps-windows-x86_64.exe" class="underline hover:text-foreground font-medium">Windows x64 (.exe) ↓</a>
    </div>
  </div>
</div>

You can also view the full release notes, changelog, and checksums on [GitHub Releases](https://github.com/FerrarioChristian/omnideps/releases/latest).

### Quick Install (macOS & Linux)

You can download and place Omnideps in your path:

```bash
# Download latest release for your platform
curl -fsSL https://raw.githubusercontent.com/FerrarioChristian/omnideps/main/install.sh | bash
```

Or manually make the binary executable:

```bash
chmod +x omnideps
sudo mv omnideps /usr/local/bin/
```

Verify the installation:

```bash
omnideps --help
```

---

## 2. Building from Source

If you want to contribute or build from source, you will need **Rust (Cargo)** and **Node.js**:

### Prerequisites
* Rust 1.80+ (2024 edition support)
* Node.js 20+ and npm

### Build Instructions

```bash
# 1. Clone the repository
git clone https://github.com/FerrarioChristian/omnideps.git
cd omnideps

# 2. Build the SvelteKit frontend (embeds into Rust binary)
cd visualizer-svelte
npm install
npm run build
cd ..

# 3. Compile the standalone binary
cargo build --release

# The compiled binary is located at ./target/release/omnideps
./target/release/omnideps --version
```
