<p align="center"><code>npm i -g @openai/adom</code><br />or <code>brew install --cask adom</code></p>

<p align="center"><strong>Adom CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
</br>
</br>If you want Adom in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/adom/ide">install in your IDE</a>
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Adom Web</strong>, go to <a href="https://chatgpt.com/adom">chatgpt.com/adom</a></p>

<p align="center">
  <img src="./.github/adom-cli-splash.png" alt="Adom CLI splash" width="80%" />
  </p>

---

## Quickstart

### Installing and running Adom CLI

Install globally with your preferred package manager. If you use npm:

```shell
npm install -g @openai/adom
```

Alternatively, if you use Homebrew:

```shell
brew install --cask adom
```

Then simply run `adom` to get started:

```shell
adom
```

If you're running into upgrade issues with Homebrew, see the [FAQ entry on brew upgrade adom](./docs/faq.md#brew-upgrade-adom-isnt-upgrading-me).

<details>
<summary>You can also go to the <a href="https://github.com/openai/adom/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `adom-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `adom-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `adom-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `adom-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `adom-x86_64-unknown-linux-musl`), so you likely want to rename it to `adom` after extracting it.

</details>

### Using Adom with your ChatGPT plan

<p align="center">
  <img src="./.github/adom-cli-login.png" alt="Adom CLI login" width="80%" />
  </p>

Run `adom` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Adom as part of your Plus, Pro, Team, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-adom-in-chatgpt).

You can also use Adom with an API key, but this requires [additional setup](./docs/authentication.md#usage-based-billing-alternative-use-an-openai-api-key). If you previously used an API key for usage-based billing, see the [migration steps](./docs/authentication.md#migrating-from-usage-based-billing-api-key). If you're having trouble with login, please comment on [this issue](https://github.com/openai/adom/issues/1243).

### Model Context Protocol (MCP)

Adom can access MCP servers. To configure them, refer to the [config docs](./docs/config.md#mcp_servers).

### Configuration

Adom CLI supports a rich set of configuration options, with preferences stored in `~/.adom/config.toml`. For full configuration options, see [Configuration](./docs/config.md).

---

### Docs & FAQ

- [**Getting started**](./docs/getting-started.md)
  - [CLI usage](./docs/getting-started.md#cli-usage)
  - [Slash Commands](./docs/slash_commands.md)
  - [Running with a prompt as input](./docs/getting-started.md#running-with-a-prompt-as-input)
  - [Example prompts](./docs/getting-started.md#example-prompts)
  - [Custom prompts](./docs/prompts.md)
  - [Memory with AGENTS.md](./docs/getting-started.md#memory-with-agentsmd)
- [**Configuration**](./docs/config.md)
  - [Example config](./docs/example-config.md)
- [**Sandbox & approvals**](./docs/sandbox.md)
- [**Authentication**](./docs/authentication.md)
  - [Auth methods](./docs/authentication.md#forcing-a-specific-auth-method-advanced)
  - [Login on a "Headless" machine](./docs/authentication.md#connecting-on-a-headless-machine)
- **Automating Adom**
  - [GitHub Action](https://github.com/openai/adom-action)
  - [TypeScript SDK](./sdk/typescript/README.md)
  - [Non-interactive mode (`adom exec`)](./docs/exec.md)
- [**Advanced**](./docs/advanced.md)
  - [Tracing / verbose logging](./docs/advanced.md#tracing--verbose-logging)
  - [Model Context Protocol (MCP)](./docs/advanced.md#model-context-protocol-mcp)
- [**Zero data retention (ZDR)**](./docs/zdr.md)
- [**Contributing**](./docs/contributing.md)
- [**Install & build**](./docs/install.md)
  - [System Requirements](./docs/install.md#system-requirements)
  - [DotSlash](./docs/install.md#dotslash)
  - [Build from source](./docs/install.md#build-from-source)
- [**FAQ**](./docs/faq.md)
- [**Open source fund**](./docs/open-source-fund.md)

---

## License

This repository is licensed under the [Apache-2.0 License](LICENSE).
