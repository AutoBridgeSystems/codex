# AutoBridge ADOM (Custom Codex)
This repo houses a custom version of Codex that has improved spec driven development flow and more.

## Features
### /spec Command
This command allows the user to create a spec using a conversational approach with ADOM. The process takes a while, but the results are worth it. It can be done with or without an initial spec.

### /index Command
This command comes with a custom Rust function that the AI can call to index a directory (and important sub-directories) in a repo using a token-friendly style. This is a great addon because if every major directory has an INDEX.md file, the AI can understand much quicker and with fewer tokens used what is going on.

### Deep Research MCP
This feature is coming soon, allows ADOM to call out to a deep research agent (in the cloud) to figure out an answer to a problem.

## Defaults
ADOM comes with some defaults enabled: 
- Internet search enabled (just ask it to search)
- Full Auto Mode enabled
- Spec driven development principles inside the system prompt
- High reasoning


## To run

1. Clone the repo.
2. Build the Docker container (takes ~20 minutes):

   ```bash
   docker build -t codex-cli .
   ```

3. Create a folder to work in (so ADOM doesn't see itself) and make it your working directory.
4. Run the container with **one** of the following commands, depending on your shell:

   * **PowerShell**

     ```powershell
     docker run --rm -it -v ${PWD}:/workspace -w /workspace codex-cli
     ```

   * **Windows Command Prompt (cmd.exe)**

     ```cmd
     docker run --rm -it -v %cd%:/workspace -w /workspace codex-cli
     ```

   * **Bash**

     ```bash
     docker run --rm -it -v "$PWD":/workspace -w /workspace codex-cli
     ```
5. You will be prompted for an API key when the session starts. Drop in an Autobridge key, and you can get started immediately. You can also set `CODEX_API_KEY` in your environment and you'll be auto logged in.
