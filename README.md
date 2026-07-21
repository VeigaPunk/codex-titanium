# Codex Titanium

An opinionated source fork of OpenAI Codex CLI, currently based on
`0.145.0-alpha.25` and published as `0.145.0-alpha.25+titanium.1`.

Titanium changes:

- Press `Delete` in the Skills menu to remove the selected skill.
- Press `Delete` in the Plugins menu to uninstall the selected plugin.
- Remove the `doctor` command and its CLI registration.
- Remove the `mcp` and `mcp-server` command implementations.
- Default `codex exec` to skipping the Git-repository check.
- Ship unrestricted Godspeed, 64-thread multi-agent v2, and DS4CC marketplace
  defaults in `titanium/config.toml`.

The Titanium preset intentionally uses `approval_policy = "never"` and
`sandbox_mode = "danger-full-access"`. It will not request permission before
commands or writes unless the user changes those settings.

```shell
install -Dm600 titanium/config.toml "${CODEX_HOME:-$HOME/.codex}/config.toml"
```

The distribution toolchain includes Bun and FNM multishell support for Pi,
XBreed, and marketplace tooling. Codex itself remains a native Rust executable.

The upstream npm package and Homebrew cask do **not** contain these changes.
Until Titanium release artifacts and its tap are published, build the Rust CLI
from `codex-rs` with Cargo.

This project preserves the upstream Apache-2.0 license and NOTICE. It is an
independent fork and is not an official OpenAI distribution.

---

## Upstream documentation

<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
<p align="center">
  <img src="https://github.com/openai/codex/blob/main/.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
</p>
</br>
If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>codex app</code> or visit <a href="https://chatgpt.com/codex?app-landing-page=true">the Codex App page</a>.
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

---

## Quickstart

### Installing and running Codex CLI

Run the following on Mac or Linux to install Codex CLI:

```shell
curl -fsSL https://chatgpt.com/codex/install.sh | sh
```

Run the following on Windows to install Codex CLI:

```shell
powershell -ExecutionPolicy ByPass -c "irm https://chatgpt.com/codex/install.ps1 | iex"
```

Codex CLI can also be installed via the following package managers:

```shell
# Install using npm
npm install -g @openai/codex
```

```shell
# Install using Homebrew
brew install --cask codex
```

Then simply run `codex` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `codex-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `codex-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `codex-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `codex-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `codex-x86_64-unknown-linux-musl`), so you likely want to rename it to `codex` after extracting it.

</details>

### Using Codex with your ChatGPT plan

Run `codex` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Codex as part of your Plus, Pro, Business, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Codex with an API key, but this requires [additional setup](https://developers.openai.com/codex/auth#sign-in-with-an-api-key).

## Docs

- [**Codex Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
