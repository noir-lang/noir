//! nargo clean command — safe, simple, testable.
//! (Implementation only; tests live in tests/clean_basics.rs)

use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

use super::{LockType, WorkspaceCommand};
use crate::errors::CliError;
use clap::Args;
use nargo::workspace::Workspace;
use nargo_toml::PackageSelection;

#[derive(Debug, Clone, Args)]
#[command(about = "Clean build artifacts (target) and optional CRS caches.")]
pub(crate) struct CleanCommand {
    #[clap(long)]
    pub target: bool,
    #[clap(long)]
    pub crs: bool,
    #[clap(long)]
    pub all: bool,
    #[clap(long, alias = "dryrun", short = 'n')]
    pub dry_run: bool,
    #[clap(long, short = 'v')]
    pub verbose: bool,
    #[clap(long)]
    pub force: bool,
}

impl WorkspaceCommand for CleanCommand {
    fn package_selection(&self) -> PackageSelection {
        PackageSelection::DefaultOrAll
    }
    fn lock_type(&self) -> LockType {
        LockType::Exclusive
    }
}

pub(crate) struct PlannedPath {
    pub label: String,
    pub path: PathBuf,
}

pub(crate) fn run(cmd: CleanCommand, workspace: Workspace) -> Result<(), CliError> {
    let (remove_target, remove_crs) = normalize_flags(&cmd);

    let target_root =
        workspace.target_dir.clone().unwrap_or_else(|| workspace.root_dir.join("target"));

    let global_crs = possible_global_crs_dir();
    let planned =
        plan_paths(&workspace, &target_root, remove_target, remove_crs, &cmd, global_crs.as_ref())?;

    if planned.is_empty() {
        if cmd.verbose || cmd.dry_run {
            println!("Nothing to clean.");
        }
        return Ok(());
    }

    if cmd.dry_run {
        println!("Dry run: would remove {} path(s):", planned.len());
        for entry in &planned {
            if cmd.verbose {
                println!(
                    "  {}: {} ({})",
                    entry.label,
                    entry.path.display(),
                    dir_size_string(&entry.path)
                );
            } else {
                println!("  {}: {}", entry.label, entry.path.display());
            }
        }
        return Ok(());
    }

    let mut errors: Vec<(PathBuf, io::Error)> = Vec::new();
    for entry in &planned {
        if cmd.verbose {
            println!("Removing {}: {}", entry.label, entry.path.display());
        }
        if let Err(e) = remove_path(&entry.path) {
            if e.kind() != io::ErrorKind::NotFound {
                errors.push((entry.path.clone(), e));
            } else if cmd.verbose {
                println!("  (Already gone) {}", entry.path.display());
            }
        }
    }

    if errors.is_empty() {
    println!("Clean complete.");
    Ok(())
    } else {
        eprintln!("Completed with {} error(s):", errors.len());
        for (p, e) in &errors {
            eprintln!("  {}: {e}", p.display());
        }
        Err(CliError::Generic(format!(
            "Failed to remove {} path(s): {}",
            errors.len(),
            errors.iter().map(|(p, _)| p.display().to_string()).collect::<Vec<_>>().join(", ")
        )))
    }
}

fn normalize_flags(cmd: &CleanCommand) -> (bool, bool) {
    let mut remove_target = cmd.target;
    let mut remove_crs = cmd.crs;
    if cmd.all {
        remove_target = true;
        remove_crs = true;
    }
    if !remove_target && !remove_crs {
        remove_target = true; // default
    }
    (remove_target, remove_crs)
}

fn plan_paths(
    workspace: &Workspace,
    target_root: &Path,
    remove_target: bool,
    remove_crs: bool,
    cmd: &CleanCommand,
    global_crs: Option<&PathBuf>,
) -> Result<Vec<PlannedPath>, CliError> {
    let mut planned = Vec::new();

    // Canonicalize the workspace root if possible; otherwise use it as-is.
    let workspace_root =
        workspace.root_dir.canonicalize().unwrap_or_else(|_| workspace.root_dir.clone());

    // Consider a path "within workspace" if:
    // - It canonically resolves under the workspace root, or
    // - It does not exist yet but lexically sits under the workspace root.
    let within_workspace = |p: &Path| {
        let abs = if p.is_absolute() { p.to_path_buf() } else { workspace.root_dir.join(p) };
        match abs.canonicalize() {
            Ok(canon) => canon.starts_with(&workspace_root),
            Err(_) => abs.starts_with(&workspace_root),
        }
    };

    if remove_target {
        if !within_workspace(target_root) {
            return Err(CliError::Generic(format!(
                "Refusing to clean target outside workspace: {}",
                target_root.display()
            )));
        }
        planned.push(PlannedPath { label: "target".into(), path: target_root.to_path_buf() });
    }

    // Only schedule local CRS subdirectories if we are not removing the entire target.
    if remove_crs && !remove_target {
        for dir in ["crs", "srs"] {
            let p = target_root.join(dir);
            if p.is_dir() && within_workspace(&p) {
                planned.push(PlannedPath { label: format!("local-{dir}"), path: p });
            }
        }
    }

    // Optionally include the global CRS directory; only with --force.
    if remove_crs {
        match (cmd.force, global_crs) {
            (true, Some(p)) if p.is_dir() => {
                planned.push(PlannedPath { label: "global-crs".into(), path: p.clone() });
            }
            (false, Some(p)) if p.is_dir() && cmd.verbose => {
                println!(
                    "Global CRS cache at {} ({}) (use --force with --crs to remove)",
                    p.display(),
                    dir_size_string(p)
                );
            }
            (true, None) if cmd.verbose => println!("(No global CRS cache found)"),
            _ => {}
        }
    }

    // Deduplicate and sort for stable output.
    let mut seen = HashSet::new();
    planned.retain(|e| seen.insert(canonical_key(&e.path)));
    planned.sort_by_key(|e| (e.label.clone(), e.path.clone()));

    Ok(planned)
}

fn remove_path(p: &Path) -> io::Result<()> {
    match fs::metadata(p) {
        Ok(md) if md.is_dir() => fs::remove_dir_all(p),
        Ok(_) => fs::remove_file(p),
        Err(e) => Err(e),
    }
}

fn canonical_key(p: &Path) -> String {
    p.canonicalize()
        .map(|c| c.to_string_lossy().into_owned())
        .unwrap_or_else(|_| p.to_string_lossy().into_owned())
}

fn home_dir() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("HOME") {
        return Some(PathBuf::from(path));
    }
    if cfg!(windows) {
        if let Some(path) = std::env::var_os("USERPROFILE") {
            return Some(PathBuf::from(path));
        }
    }
    None
}

fn dir_size_string(path: &Path) -> String {
    fn walk(p: &Path, acc: &mut u64) {
        if let Ok(md) = fs::symlink_metadata(p) {
            if md.is_file() {
                *acc += md.len();
            } else if md.is_dir() {
                if let Ok(read) = fs::read_dir(p) {
                    for entry in read.flatten() {
                        let child = entry.path();
                        if let Ok(ft) = entry.file_type() {
                            if ft.is_symlink() {
                                continue;
                            }
                        }
                        walk(&child, acc);
                    }
                }
            }
        }
    }
    if !path.exists() {
        return "0 B".into();
    }
    let mut total = 0;
    walk(path, &mut total);
    human_size(total)
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if bytes == 0 {
        return "0 B".into();
    }
    let mut val = bytes as f64;
    let mut idx = 0usize;
    while val >= 1024.0 && idx < UNITS.len() - 1 {
        val /= 1024.0;
        idx += 1;
    }
    if idx == 0 { format!("{bytes} {}", UNITS[idx]) } else { format!("{:.2} {}", val, UNITS[idx]) }
}
