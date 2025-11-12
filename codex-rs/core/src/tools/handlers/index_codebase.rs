use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::Deserialize;
use tokio::task;

use crate::function_tool::FunctionCallError;
use crate::tools::context::{ToolInvocation, ToolOutput, ToolPayload};
use crate::tools::registry::{ToolHandler, ToolKind};

const DEFAULT_EXCLUDES: &[&str] = &[
    "node_modules/**",
    "**/node_modules/**",
    "node",
    "node/**",
    "**/node/**",
    ".git",
    ".git/**",
    "**/.git/**",
    ".venv",
    ".venv/**",
    "**/.venv/**",
    "venv",
    "venv/**",
    "**/venv/**",
    "env",
    "env/**",
    "**/env/**",
    ".tox",
    ".tox/**",
    ".cache",
    ".cache/**",
    ".idea",
    ".idea/**",
    ".vscode",
    ".vscode/**",
    "__pycache__",
    "__pycache__/**",
    "**/__pycache__/**",
    "dist/**",
    "**/dist/**",
    "build/**",
    "**/build/**",
    "storage/files/**",
];

const PREVIEW_LINE_LIMIT: usize = 40;

#[derive(Debug, Deserialize)]
struct IndexCodebaseArgs {
    root: String,
    #[serde(default)]
    output_path: Option<String>,
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    max_depth: Option<usize>,
}

pub struct IndexCodebaseHandler;

#[async_trait]
impl ToolHandler for IndexCodebaseHandler {
    fn kind(&self) -> ToolKind {
        ToolKind::Function
    }

    async fn handle(&self, invocation: ToolInvocation) -> Result<ToolOutput, FunctionCallError> {
        let ToolInvocation { payload, turn, .. } = invocation;
        let arguments = match payload {
            ToolPayload::Function { arguments } => arguments,
            _ => {
                return Err(FunctionCallError::RespondToModel(
                    "index_codebase handler received unsupported payload".to_string(),
                ));
            }
        };

        let args: IndexCodebaseArgs = serde_json::from_str(&arguments).map_err(|err| {
            FunctionCallError::RespondToModel(format!("failed to parse function arguments: {err}"))
        })?;

        let IndexCodebaseArgs {
            root,
            output_path,
            exclude,
            max_depth,
        } = args;

        let depth_limit = match max_depth {
            Some(0) => {
                return Err(FunctionCallError::RespondToModel(
                    "max_depth must be at least 1 when provided".to_string(),
                ));
            }
            Some(v) => v,
            None => usize::MAX,
        };

        let resolved_root = turn.resolve_path(Some(root.clone()));
        let canonical_root = dunce::canonicalize(&resolved_root).map_err(|err| {
            FunctionCallError::RespondToModel(format!(
                "root not found or inaccessible: {} ({err})",
                resolved_root.display()
            ))
        })?;
        if !canonical_root.is_dir() {
            return Err(FunctionCallError::RespondToModel(format!(
                "root must be a directory: {}",
                canonical_root.display()
            )));
        }

        let resolved_output = if let Some(path) = output_path {
            let candidate = PathBuf::from(&path);
            let resolved = if candidate.is_absolute() {
                candidate
            } else {
                canonical_root.join(candidate)
            };
            if !resolved.starts_with(&canonical_root) {
                return Err(FunctionCallError::RespondToModel(format!(
                    "output_path must be inside the root directory: {path}"
                )));
            }
            Some(resolved)
        } else {
            None
        };

        let blocking_root = canonical_root.clone();
        let blocking_output = resolved_output.clone();
        let blocking_excludes = exclude.clone();

        let join_result = task::spawn_blocking(move || {
            index_codebase_blocking(
                blocking_root,
                blocking_output,
                blocking_excludes,
                depth_limit,
            )
        })
        .await
        .map_err(|err| {
            FunctionCallError::Fatal(format!(
                "index_codebase task failed or was cancelled: {err}"
            ))
        })?;

        let index_result = join_result?;
        let mut content = String::new();
        let _ = writeln!(
            content,
            "Indexed directory: {}",
            canonical_root.display()
        );
        if let Some(path) = index_result.output_path.as_ref() {
            let _ = writeln!(content, "Markdown written to: {}", path.display());
        }
        let _ = writeln!(
            content,
            "Entries: {} directories, {} files ({} lines in listing)",
            index_result.directories,
            index_result.files,
            index_result.total_lines
        );

        if index_result.total_lines == 0 {
            let _ = writeln!(content, "\nDirectory contained no indexable entries.");
        } else {
            let preview_header = format!(
                "\nPreview (showing {} of {} lines):",
                index_result.preview_lines,
                index_result.total_lines
            );
            let _ = writeln!(content, "{preview_header}");
            content.push_str(&index_result.preview);
        }

        Ok(ToolOutput::Function {
            content,
            content_items: None,
            success: Some(true),
        })
    }
}

struct NodeInfo {
    name: String,
    is_dir: bool,
}

#[derive(Clone)]
struct FileMeta {
    name: String,
    rel_path: PathBuf,
    is_dir: bool,
    children: Vec<FileMeta>,
}

#[derive(Default)]
struct IndexStats {
    directories: usize,
    files: usize,
}

struct IndexResult {
    listing: String,
    preview: String,
    preview_lines: usize,
    total_lines: usize,
    directories: usize,
    files: usize,
    output_path: Option<PathBuf>,
}

fn index_codebase_blocking(
    root: PathBuf,
    output_path: Option<PathBuf>,
    extra_excludes: Vec<String>,
    depth_limit: usize,
) -> Result<IndexResult, FunctionCallError> {
    let matcher = build_gitignore(&root, &extra_excludes)?;
    let (child_map, info_map) = collect_entries(&root, &matcher, depth_limit)?;
    let mut stats = IndexStats::default();
    let mut root_children = child_map
        .get(&PathBuf::from("."))
        .cloned()
        .unwrap_or_default();
    sort_children(&mut root_children, &info_map);

    let mut nodes = Vec::new();
    for child in root_children {
        nodes.push(build_tree(&child, &child_map, &info_map, &mut stats));
    }

    let listing = render_ls_r(&nodes);
    let total_lines = if listing.is_empty() {
        0
    } else {
        listing.lines().count()
    };
    let preview_lines: Vec<&str> = listing.lines().take(PREVIEW_LINE_LIMIT).collect();
    let mut preview = preview_lines.join("\n");
    if total_lines > preview_lines.len() {
        if !preview.is_empty() {
            preview.push('\n');
        }
        preview.push_str("... truncated ...");
    }

    if let Some(path) = output_path.as_ref() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                FunctionCallError::Fatal(format!(
                    "failed to create output directory {}: {err}",
                    parent.display()
                ))
            })?;
        }
        write_markdown(&listing, &root, path)?;
    }

    Ok(IndexResult {
        listing,
        preview,
        preview_lines: preview_lines.len(),
        total_lines,
        directories: stats.directories,
        files: stats.files,
        output_path,
    })
}

fn build_gitignore(root: &Path, extra: &[String]) -> Result<Gitignore, FunctionCallError> {
    let mut builder = GitignoreBuilder::new(root);
    let gitignore_path = root.join(".gitignore");
    if gitignore_path.exists() {
        builder.add(&gitignore_path).map_err(|err| {
            FunctionCallError::Fatal(format!(
                "failed to parse .gitignore at {}: {err}",
                gitignore_path.display()
            ))
        })?;
    }

    for pattern in expanded_patterns(DEFAULT_EXCLUDES.iter().copied()) {
        builder
            .add_line(None, &pattern)
            .map_err(|err| FunctionCallError::Fatal(format!("invalid exclude pattern {pattern}: {err}")))?;
    }
    for pattern in expanded_patterns(extra.iter().map(String::as_str)) {
        builder
            .add_line(None, &pattern)
            .map_err(|err| FunctionCallError::Fatal(format!("invalid exclude pattern {pattern}: {err}")))?;
    }

    builder.build().map_err(|err| {
        FunctionCallError::Fatal(format!(
            "failed to build ignore matcher for {}: {err}",
            root.display()
        ))
    })
}

fn expanded_patterns<'a, I>(patterns: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut out = Vec::new();
    for pattern in patterns {
        out.push(pattern.to_string());
        if let Some(stripped) = pattern.strip_suffix("/**") {
            if !stripped.is_empty() {
                out.push(format!("{stripped}/"));
                out.push(stripped.to_string());
            }
        }
    }
    out
}

fn collect_entries(
    root: &Path,
    matcher: &Gitignore,
    depth_limit: usize,
) -> Result<(HashMap<PathBuf, Vec<PathBuf>>, HashMap<PathBuf, NodeInfo>), FunctionCallError> {
    let mut child_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut info_map: HashMap<PathBuf, NodeInfo> = HashMap::new();
    child_map.insert(PathBuf::from("."), Vec::new());
    info_map.insert(
        PathBuf::from("."),
        NodeInfo {
            name: ".".to_string(),
            is_dir: true,
        },
    );

    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((current, depth)) = stack.pop() {
        let rel_dir = if current == root {
            PathBuf::from(".")
        } else {
            match current.strip_prefix(root) {
                Ok(path) => path.to_path_buf(),
                Err(err) => {
                    return Err(FunctionCallError::Fatal(format!(
                        "failed to compute relative path for {}: {err}",
                        current.display()
                    )));
                }
            }
        };

        let read_dir = match fs::read_dir(&current) {
            Ok(iter) => iter,
            Err(err) => {
                if err.kind() == io::ErrorKind::PermissionDenied {
                    continue;
                }
                return Err(FunctionCallError::Fatal(format!(
                    "failed to read directory {}: {err}",
                    current.display()
                )));
            }
        };

        let mut entries: Vec<(String, String, fs::DirEntry)> = Vec::new();
        for entry_result in read_dir {
            match entry_result {
                Ok(entry) => {
                    let name_lossy = entry.file_name().to_string_lossy().to_string();
                    let sort_key = name_lossy.to_lowercase();
                    entries.push((sort_key, name_lossy, entry));
                }
                Err(err) => {
                    if err.kind() == io::ErrorKind::PermissionDenied {
                        continue;
                    }
                    return Err(FunctionCallError::Fatal(format!(
                        "failed to read entry in {}: {err}",
                        current.display()
                    )));
                }
            }
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        let mut children = Vec::new();
        for (_, original_name, entry) in entries {
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(err) => {
                    return Err(FunctionCallError::Fatal(format!(
                        "failed to read file type for {}: {err}",
                        entry.path().display()
                    )));
                }
            };
            if file_type.is_symlink() {
                continue;
            }

            let abs_path = entry.path();
            let is_dir = file_type.is_dir();
            if matcher
                .matched_path_or_any_parents(&abs_path, is_dir)
                .is_ignore()
            {
                continue;
            }

            let rel_child = match abs_path.strip_prefix(root) {
                Ok(path) => path.to_path_buf(),
                Err(err) => {
                    return Err(FunctionCallError::Fatal(format!(
                        "failed to compute relative path for {}: {err}",
                        abs_path.display()
                    )));
                }
            };

            children.push(rel_child.clone());
            info_map.insert(
                rel_child.clone(),
                NodeInfo {
                    name: original_name.clone(),
                    is_dir,
                },
            );

            if is_dir {
                child_map.entry(rel_child.clone()).or_default();
            }

            let child_depth = depth.saturating_add(1);
            if is_dir && child_depth < depth_limit {
                stack.push((abs_path, child_depth));
            }
        }
        if !children.is_empty() {
            let entry = child_map.entry(rel_dir).or_default();
            entry.extend(children);
        }
    }

    Ok((child_map, info_map))
}

fn build_tree(
    path: &PathBuf,
    child_map: &HashMap<PathBuf, Vec<PathBuf>>,
    info_map: &HashMap<PathBuf, NodeInfo>,
    stats: &mut IndexStats,
) -> FileMeta {
    let info = &info_map[path];
    if info.is_dir {
        stats.directories = stats.directories.saturating_add(1);
    } else {
        stats.files = stats.files.saturating_add(1);
    }

    let mut children = child_map.get(path).cloned().unwrap_or_default();
    sort_children(&mut children, info_map);
    let mut child_nodes = Vec::with_capacity(children.len());
    for child in children {
        child_nodes.push(build_tree(&child, child_map, info_map, stats));
    }

    FileMeta {
        name: info.name.clone(),
        rel_path: path.clone(),
        is_dir: info.is_dir,
        children: child_nodes,
    }
}

fn sort_children(children: &mut [PathBuf], info_map: &HashMap<PathBuf, NodeInfo>) {
    children.sort_by_key(|path| {
        let entry = &info_map[path];
        (entry.name.to_lowercase(), entry.name.clone())
    });
}

fn render_ls_r(nodes: &[FileMeta]) -> String {
    let mut lines = Vec::new();
    add_section(".", nodes, &mut lines);
    for node in nodes {
        if node.is_dir {
            render_dir(node, &mut lines);
        }
    }
    if lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

fn add_section(title: &str, entries: &[FileMeta], lines: &mut Vec<String>) {
    lines.push(format!("{title}:"));
    for entry in entries {
        lines.push(entry.name.clone());
    }
    lines.push(String::new());
}

fn render_dir(dir: &FileMeta, lines: &mut Vec<String>) {
    let section_title = format!("./{}", path_to_posix(&dir.rel_path));
    add_section(&section_title, &dir.children, lines);
    for child in &dir.children {
        if child.is_dir {
            render_dir(child, lines);
        }
    }
}

fn write_markdown(listing: &str, root: &Path, output_path: &Path) -> Result<(), FunctionCallError> {
    let root_name = root
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| ".".to_string());
    let mut doc = String::new();
    let _ = writeln!(doc, "# Index - {root_name}\n");
    doc.push_str("```text\n");
    doc.push_str(listing);
    if !listing.ends_with('\n') && !listing.is_empty() {
        doc.push('\n');
    }
    doc.push_str("```\n");
    fs::write(output_path, doc).map_err(|err| {
        FunctionCallError::Fatal(format!(
            "failed to write index file {}: {err}",
            output_path.display()
        ))
    })
}

fn path_to_posix(path: &Path) -> String {
    let mut parts = Vec::new();
    for component in path.iter() {
        let part = component.to_string_lossy();
        if !part.is_empty() {
            parts.push(part.to_string());
        }
    }
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;

    #[test]
    fn collects_and_renders_index() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path();
        fs::create_dir(root.join("src")).expect("create src");
        fs::create_dir(root.join("docs")).expect("create docs");
        fs::write(root.join("README.md"), "readme").expect("create README");
        fs::write(root.join("src/lib.rs"), "lib").expect("create lib");
        fs::write(root.join("docs/guide.md"), "guide").expect("create guide");

        let result = index_codebase_blocking(
            root.to_path_buf(),
            None,
            Vec::new(),
            usize::MAX,
        )
        .expect("index directory");

        assert_eq!(result.directories, 2);
        assert_eq!(result.files, 3);
        assert!(result.listing.contains("./docs"));
        assert!(result.listing.contains("./src"));
        assert!(result.listing.contains("README.md"));
    }

    #[test]
    fn respects_max_depth_and_excludes() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path();
        fs::create_dir(root.join("dir_a")).expect("create dir_a");
        fs::create_dir(root.join("dir_a/nested")).expect("create nested");
        fs::write(root.join("dir_a/file.txt"), "file").expect("create file");
        fs::create_dir(root.join("node_modules")).expect("create node_modules");
        fs::write(root.join("node_modules/pkg.js"), "pkg").expect("create pkg");

        let result = index_codebase_blocking(
            root.to_path_buf(),
            None,
            Vec::new(),
            2,
        )
        .expect("index directory");

        assert!(
            !result.listing.contains("node_modules"),
            "default excludes should skip node_modules"
        );
        assert!(
            !result.listing.contains("nested"),
            "max_depth should truncate traversal"
        );
    }

    #[test]
    fn writes_markdown_output() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path();
        fs::write(root.join("file.txt"), "content").expect("create file");
        let output = root.join("docs/index.md");

        let result = index_codebase_blocking(
            root.to_path_buf(),
            Some(output.clone()),
            Vec::new(),
            usize::MAX,
        )
        .expect("index directory");

        assert_eq!(result.output_path.as_ref(), Some(&output));
        let written = fs::read_to_string(&output).expect("read markdown");
        assert!(written.starts_with("# Index - "));
        assert!(written.contains("```text"));
    }
}
