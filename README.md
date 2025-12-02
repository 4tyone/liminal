<p align="center">
  <img src="docs/icon.png" alt="Liminal Logo" width="128" height="128">
</p>

<h1 align="center">Liminal</h1>

<p align="center">
  <strong>Build Your Custom Learning Experience</strong>
</p>

<p align="center">
  Liminal is a desktop app that generates personalized, book-quality learning materials on any topic. Ask questions, get instant explanations, and master concepts faster than ever.
</p>

<p align="center">
  <a href="https://liminal.melshakobyan.com">Website</a> •
  <a href="https://github.com/4tyone/liminal/releases">Download</a>
</p>

<p align="center">
  <a href="https://liminal.melshakobyan.com">
    <strong>Watch the demo on the website →</strong>
  </a>
</p>

## Features

- **Learn any topic** - From quantum physics to cooking, generate comprehensive learning materials on any subject
- **Inline Q&A** - Highlight any text and ask questions. Get contextual explanations that blend into your content
- **Bring your own API key** - Works with any OpenAI-compatible API (OpenAI, OpenRouter, custom endpoints)
- **Local-first** - Your data stays on your device. Your API key is stored locally and never shared
- **Export to PDF** - Export your learning guides for offline reading or sharing

## Download

Download the latest release from the [Releases page](https://github.com/4tyone/liminal/releases).

- **Mac (Apple Silicon)**: `Liminal_x.x.x_aarch64.dmg`
- **Mac (Intel)**: `Liminal_x.x.x_x64.dmg`

## Getting Started

1. Download and install Liminal
2. Open Settings and add your API key (OpenAI, OpenRouter, or custom endpoint)
3. Create a new project and enter a topic you want to learn
4. Start learning!

## Supported API Providers

Liminal works with any OpenAI-compatible API:

- **OpenAI** - `https://api.openai.com/v1`
- **OpenRouter** - `https://openrouter.ai/api/v1` (access Claude, Gemini, and more)
- **Custom endpoints** - Any OpenAI-compatible API

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Setup

```bash
# Clone the repository
git clone https://github.com/4tyone/liminal.git
cd liminal

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev
```

### Building

```bash
# Build for production
pnpm tauri build
```

### Project Structure

```
liminal/
├── src/                  # Frontend (HTML, CSS, JavaScript)
│   ├── js/              # JavaScript modules
│   │   ├── pages/       # Page components
│   │   ├── components/  # Reusable UI components
│   │   └── api.js       # Tauri API wrapper
│   ├── input.css        # Source CSS
│   └── index.html       # Main HTML
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── commands/    # Tauri commands
│   │   ├── services/    # Business logic
│   │   └── models/      # Data models
│   └── tauri.conf.json  # Tauri configuration
└── scripts/             # Build and release scripts
```

## Releasing

To create a new release:

```bash
# Update version and build release artifacts
./scripts/release.sh 0.3.0
```

This will:
1. Update version in `tauri.conf.json` and `Cargo.toml`
2. Build for both Apple Silicon and Intel
3. Create signed DMG files
4. Generate `latest.json` for auto-updates
5. Create a GitHub release with all artifacts

### Requirements for releasing

- Tauri signing key at `~/.tauri/liminal.key`
- GitHub CLI (`gh`) authenticated

## License

MIT License - see [LICENSE](LICENSE) for details.

## Links

- [Website](https://liminal.melshakobyan.com)
- [Releases](https://github.com/4tyone/liminal/releases)
