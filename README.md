# Skills Pal

Skills Pal is a lightweight, AI-powered CLI tool designed to help you tackle technical debt. By analyzing your project's structure, it recommends the best open-source plugins and tools tailored to your specific stack. 

Instead of getting lost in endless documentation or ecosystem fatigue, let the AI suggest exactly what you need to clean up, format, or optimize your codebase.

## How it works

The project is split into two parts:
- **The CLI**: A fast Rust binary that runs locally. It scans your project's file extensions and asks an LLM (OpenAI or Mistral) for tailored recommendations.
- **The Registry (Backend)**: An Axum server running on Railway. It constantly scrapes GitHub for community-made plugins tagged with `skills-pal-plugin` and serves them to the CLI via a low-latency Postgres/RAM cache.

## Installation

You don't need to install Rust or build from source. You can grab the pre-compiled binary directly.

**Mac / Linux**
```bash
curl -fsSL https://raw.githubusercontent.com/Julien-Bui/skills_pal/main/install.sh | bash
```

**Windows (PowerShell)**
```powershell
iwr https://raw.githubusercontent.com/Julien-Bui/skills_pal/main/install.ps1 -useb | iex
```

## Usage

1. **Initialize the config** in any of your projects:
   ```bash
   skills_pal init
   ```
   *(This creates a local `.skillspal.toml` where you can set your API key).*

2. **Get recommendations**:
   ```bash
   skills_pal recom
   ```
   The CLI will analyze your current directory, contact the registry for available plugins, and ask the AI for the best matches.

3. **Check your current setup**:
   ```bash
   skills_pal scan
   ```

## Creating a Plugin for the Community

Have you built a script, a linter, or a CLI tool that you want Skills Pal to recommend to others?

Simply add the `skills-pal-plugin` topic to your public GitHub repository. Our backend scraper runs every 12 hours, picks up new repositories with this tag, and automatically adds them to the global registry so the AI can start suggesting your tool to developers who need it.

## Architecture

- Written entirely in **Rust** for zero-overhead performance.
- Global DoS protection on the API registry (`tower::limit`).
- 100% immune to SQL injections thanks to `sqlx` parameter binding.
- Privacy-first: Your code stays local, only the project stack context is sent to the LLM.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.
