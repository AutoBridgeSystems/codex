# Authentication

## Default: OpenAI API key

Codex now prompts for an OpenAI API key the first time you run it. Paste the key when asked and it will be stored securely in `~/.codex/auth.json` (or `C:\Users\<you>\.codex\auth.json` on Windows). On subsequent runs the CLI reuses the saved key, so you only have to paste it once.

To automate the login, pipe the key through stdin instead:

```shell
printenv OPENAI_API_KEY | codex login --with-api-key
# or
codex login --with-api-key < my_key.txt
```

This key must, at minimum, have write access to the Responses API.

## Switching to ChatGPT login

Need to use your ChatGPT Plus/Pro/Team/Edu plan instead of an API key? Opt into the browser-based flow:

1. Add `forced_login_method = "chatgpt"` to `~/.codex/config.toml` (or run Codex with `-c forced_login_method="chatgpt"`).
2. Delete `~/.codex/auth.json` if it already contains an API key.
3. Run `codex login` (or start the TUI) and select **Sign in with ChatGPT**. The CLI opens `http://localhost:1455/...` in your browser so you can complete the OAuth flow.

Remove the config entry (or set it back to `"api"`) to return to the default API-key prompt.

## Connecting on a "Headless" Machine

The steps below are only required for the ChatGPT login flow. For API keys you can simply run `codex login --with-api-key` inside the container/VM and paste the key.

When using ChatGPT login on a remote or containerized environment, the local browser cannot reach `localhost:1455` on the remote host, so use one of the following workarounds:

### Authenticate locally and copy your credentials to the "headless" machine

The easiest solution is likely to run through the `codex login` process on your local machine such that `localhost:1455` _is_ accessible in your web browser. When you complete the authentication process, an `auth.json` file should be available at `$CODEX_HOME/auth.json` (on Mac/Linux, `$CODEX_HOME` defaults to `~/.codex` whereas on Windows, it defaults to `%USERPROFILE%\\.codex`).

Because the `auth.json` file is not tied to a specific host, once you complete the authentication flow locally, you can copy the `$CODEX_HOME/auth.json` file to the headless machine and then `codex` should "just work" on that machine. Note to copy a file to a Docker container, you can do:

```shell
# substitute MY_CONTAINER with the name or id of your Docker container:
CONTAINER_HOME=$(docker exec MY_CONTAINER printenv HOME)
docker exec MY_CONTAINER mkdir -p "$CONTAINER_HOME/.codex"
docker cp auth.json MY_CONTAINER:"$CONTAINER_HOME/.codex/auth.json"
```

whereas if you are `ssh`'d into a remote machine, you likely want to use [`scp`](https://en.wikipedia.org/wiki/Secure_copy_protocol):

```shell
ssh user@remote 'mkdir -p ~/.codex'
scp ~/.codex/auth.json user@remote:~/.codex/auth.json
```

or try this one-liner:

```shell
ssh user@remote 'mkdir -p ~/.codex && cat > ~/.codex/auth.json' < ~/.codex/auth.json
```

### Connecting through VPS or remote

If you run Codex on a remote machine (VPS/server) without a local browser, the login helper starts a server on `localhost:1455` on the remote host. To complete login in your local browser, forward that port to your machine before starting the login flow:

```bash
# From your local machine
ssh -L 1455:localhost:1455 <user>@<remote-host>
```

Then, in that SSH session, run `codex login` (or start the TUI) and select "Sign in with ChatGPT". When prompted, open the printed URL (it will be `http://localhost:1455/...`) in your local browser. The traffic will be tunneled to the remote server.
