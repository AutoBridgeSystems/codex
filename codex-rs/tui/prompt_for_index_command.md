Create a reusable filesystem index for this repository using the `index_codebase` tool. The goal is to leave behind Markdown maps that future agents (and humans) can open to understand the project layout quickly.

Workplan

1. Determine the absolute repository root (use the configured working directory or run `pwd`).  
2. Run `index_codebase` with:
   - `root`: the repository root path  
   - `output_path`: `ROOT_INDEX.md`  
   - `max_depth`: `3` (capture top-level directories and their immediate children)  
3. Inspect the generated listing to identify “major” directories worth deeper coverage (skip transient folders such as build artifacts, node modules, caches, or generated output).  
4. For each major directory, call `index_codebase` again with:
   - `root`: the absolute path to that directory  
   - `output_path`: `<directory-name>/<directory-name>_INDEX.md` 
   - `max_depth`: `3` so we do not go more than two layers deep overall  
5. If a major directory contains a noteworthy immediate subdirectory (second layer) that merits its own map, run the tool one additional time for that subdirectory, still stopping at depth 3.  
6. Finish by summarizing which maps were written, highlighting any directories intentionally skipped, and confirming the files that now exist.

Tool notes

- The tool already ignores common noise (e.g., `node_modules`, virtualenvs, caches).  
- Extra excludes can be supplied via the `exclude` argument when needed (for example, to skip very large generated trees).  
- The Markdown output uses an `ls -R` style code block; keep file paths relative to the repository root.
- Don't create an INDEX or docs folder, place the (directory_name)_index.md files for each directory at the root of each respective directory. 

Deliverable

- A brief summary explaining the coverage.  
- Links/paths to the newly generated Markdown maps (`docs/INDEX.md` and any `docs/index/*.md` files).  
- Any follow-up recommendations if large portions of the project were intentionally omitted.
