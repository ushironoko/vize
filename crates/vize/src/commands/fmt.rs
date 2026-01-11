//! Format command - High-performance Vue SFC formatting using vize_glyph

use clap::Args;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use vize_glyph::{format_sfc_with_allocator, Allocator, FormatOptions};

#[derive(Args)]
pub struct FmtArgs {
    /// Glob pattern(s) to match .vue files
    #[arg(default_value = "./**/*.vue")]
    pub patterns: Vec<String>,

    /// Check formatting without writing (exit with error if files need formatting)
    #[arg(long)]
    pub check: bool,

    /// Write formatted output to files
    #[arg(short, long)]
    pub write: bool,

    /// Config file path
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Use single quotes instead of double quotes
    #[arg(long)]
    pub single_quote: bool,

    /// Print width (line length) for formatting
    #[arg(long, default_value = "100")]
    pub print_width: u32,

    /// Number of spaces per indentation level
    #[arg(long, default_value = "2")]
    pub tab_width: u8,

    /// Use tabs instead of spaces for indentation
    #[arg(long)]
    pub use_tabs: bool,

    /// Do not print semicolons at the ends of statements
    #[arg(long)]
    pub no_semi: bool,
}

pub fn run(args: FmtArgs) {
    let options = build_format_options(&args);

    // Collect files to format
    let files: Vec<PathBuf> = collect_files(&args.patterns);

    if files.is_empty() {
        eprintln!("No .vue files found matching the patterns");
        return;
    }

    eprintln!("Found {} .vue file(s)", files.len());

    let has_errors = AtomicBool::new(false);
    let files_changed = AtomicUsize::new(0);
    let files_unchanged = AtomicUsize::new(0);
    let files_errored = AtomicUsize::new(0);

    // Process files in parallel, each thread gets its own allocator for maximum performance
    files.par_iter().for_each(|path| {
        // Create per-thread allocator with estimated capacity
        let allocator = Allocator::with_capacity(64 * 1024); // 64KB initial capacity

        match process_file(path, &options, &allocator, args.check, args.write) {
            Ok(changed) => {
                if changed {
                    files_changed.fetch_add(1, Ordering::Relaxed);
                    if args.check {
                        has_errors.store(true, Ordering::Relaxed);
                    }
                } else {
                    files_unchanged.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(err) => {
                eprintln!("Error formatting {}: {}", path.display(), err);
                files_errored.fetch_add(1, Ordering::Relaxed);
                has_errors.store(true, Ordering::Relaxed);
            }
        }
    });

    // Print summary
    let changed = files_changed.load(Ordering::Relaxed);
    let unchanged = files_unchanged.load(Ordering::Relaxed);
    let errored = files_errored.load(Ordering::Relaxed);

    eprintln!();
    if args.check {
        eprintln!("Checked {} file(s)", files.len());
        if changed > 0 {
            eprintln!("  {} file(s) would be reformatted", changed);
        }
        if unchanged > 0 {
            eprintln!("  {} file(s) already formatted", unchanged);
        }
    } else if args.write {
        eprintln!("Formatted {} file(s)", files.len());
        if changed > 0 {
            eprintln!("  {} file(s) reformatted", changed);
        }
        if unchanged > 0 {
            eprintln!("  {} file(s) unchanged", unchanged);
        }
    } else {
        eprintln!(
            "Checked {} file(s) (use --write to apply changes)",
            files.len()
        );
        if changed > 0 {
            eprintln!("  {} file(s) would be reformatted", changed);
        }
    }

    if errored > 0 {
        eprintln!("  {} file(s) had errors", errored);
    }

    if has_errors.load(Ordering::Relaxed) {
        std::process::exit(1);
    }
}

#[inline]
fn build_format_options(args: &FmtArgs) -> FormatOptions {
    FormatOptions {
        print_width: args.print_width,
        tab_width: args.tab_width,
        use_tabs: args.use_tabs,
        semi: !args.no_semi,
        single_quote: args.single_quote,
        ..Default::default()
    }
}

fn collect_files(patterns: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for pattern in patterns {
        // Use ignore crate to walk directories respecting .gitignore
        let walker = WalkBuilder::new(".")
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "vue") {
                // Check if path matches pattern (simple glob matching)
                if matches_pattern(path, pattern) {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    // Remove duplicates
    files.sort();
    files.dedup();

    files
}

#[inline]
fn matches_pattern(path: &std::path::Path, pattern: &str) -> bool {
    // Simple pattern matching - for now just check if it's a .vue file
    if pattern == "./**/*.vue" {
        return true;
    }

    // Check if pattern matches the file path
    let path_str = path.to_string_lossy();

    if pattern.contains('*') {
        // Simple glob: **/*.vue matches any .vue file
        if pattern.ends_with("*.vue") {
            return path_str.ends_with(".vue");
        }
        true
    } else {
        // Exact match
        path_str.contains(pattern)
    }
}

#[inline]
fn process_file(
    path: &PathBuf,
    options: &FormatOptions,
    allocator: &Allocator,
    check: bool,
    write: bool,
) -> Result<bool, String> {
    // Read the file
    let source = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Format the source using the provided allocator
    let result = format_sfc_with_allocator(&source, options, allocator)
        .map_err(|e| format!("Format error: {}", e))?;

    if result.changed {
        if check {
            // In check mode, just report that the file would change
            eprintln!("Would reformat: {}", path.display());
        } else if write {
            // Write the formatted output
            fs::write(path, &result.code).map_err(|e| format!("Failed to write file: {}", e))?;
            eprintln!("Reformatted: {}", path.display());
        } else {
            // Print the diff or formatted output
            eprintln!("Would reformat: {}", path.display());
        }
    }

    Ok(result.changed)
}
