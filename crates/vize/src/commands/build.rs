//! Build command - Compile Vue SFC files

use clap::{Args, ValueEnum};
use ignore::Walk;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use vize_atelier_sfc::{
    compile_sfc, parse_sfc, ScriptCompileOptions, SfcCompileOptions, SfcParseOptions,
    StyleCompileOptions, TemplateCompileOptions,
};

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    /// Output compiled JavaScript
    #[default]
    Js,
    /// Output JSON with code and metadata
    Json,
    /// Only show statistics (no output)
    Stats,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum ScriptExtension {
    /// Preserve original script language extension (.ts -> .ts, .tsx -> .tsx, .jsx -> .jsx)
    Preserve,
    /// Downcompile all scripts to JavaScript (.ts -> .js, .tsx -> .js, .jsx -> .js)
    #[default]
    Downcompile,
}

#[derive(Args, Default)]
pub struct BuildArgs {
    /// Glob pattern(s) to match .vue files (default: ./**/*.vue)
    #[arg(default_value = "./**/*.vue")]
    pub patterns: Vec<String>,

    /// Output directory (default: ./dist)
    #[arg(short, long, default_value = "./dist")]
    pub output: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value = "js")]
    pub format: OutputFormat,

    /// Enable SSR mode
    #[arg(long)]
    pub ssr: bool,

    /// Script extension handling: 'preserve' keeps original extension (.ts/.tsx/.jsx), 'downcompile' converts to .js
    #[arg(long, value_enum, default_value = "downcompile")]
    pub script_ext: ScriptExtension,

    /// Number of threads (default: number of CPUs)
    #[arg(short = 'j', long)]
    pub threads: Option<usize>,

    /// Show timing profile breakdown
    #[arg(long)]
    pub profile: bool,

    /// Continue on errors
    #[arg(long)]
    pub continue_on_error: bool,
}

#[derive(Debug)]
struct CompileStats {
    #[allow(dead_code)]
    total_files: usize,
    success: AtomicUsize,
    failed: AtomicUsize,
    #[allow(dead_code)]
    total_bytes: AtomicUsize,
    #[allow(dead_code)]
    output_bytes: AtomicUsize,
}

impl CompileStats {
    fn new(total_files: usize) -> Self {
        Self {
            total_files,
            success: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            total_bytes: AtomicUsize::new(0),
            output_bytes: AtomicUsize::new(0),
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct CompileOutput {
    filename: String,
    code: String,
    css: Option<String>,
    errors: Vec<String>,
    warnings: Vec<String>,
    script_lang: String,
}

pub fn run(args: BuildArgs) {
    let start = Instant::now();

    if let Some(threads) = args.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .expect("Failed to configure thread pool");
    }

    let files = collect_files(&args.patterns);

    if files.is_empty() {
        eprintln!("No .vue files found matching the patterns");
        std::process::exit(1);
    }

    let stats = CompileStats::new(files.len());
    let collect_elapsed = start.elapsed();

    if args.profile {
        eprintln!(
            "Found {} files in {:.4}s. Compiling using {} threads...",
            files.len(),
            collect_elapsed.as_secs_f64(),
            rayon::current_num_threads()
        );
    }

    let compile_start = Instant::now();
    let results: Vec<_> = files
        .par_iter()
        .map(|path| {
            let source_size = fs::metadata(path).map(|m| m.len() as usize).unwrap_or(0);
            stats.total_bytes.fetch_add(source_size, Ordering::Relaxed);

            match compile_file(path, args.ssr, args.script_ext) {
                Ok(output) => {
                    stats.success.fetch_add(1, Ordering::Relaxed);
                    stats
                        .output_bytes
                        .fetch_add(output.code.len(), Ordering::Relaxed);

                    if args.profile && !output.errors.is_empty() {
                        for err in &output.errors {
                            eprintln!("  {} warning: {}", path.display(), err);
                        }
                    }

                    Some((path.clone(), output))
                }
                Err(e) => {
                    stats.failed.fetch_add(1, Ordering::Relaxed);
                    eprintln!("Error compiling {}: {}", path.display(), e);

                    if !args.continue_on_error {
                        std::process::exit(1);
                    }

                    None
                }
            }
        })
        .collect();
    let compile_elapsed = compile_start.elapsed();

    let io_start = Instant::now();
    match args.format {
        OutputFormat::Stats => {}
        OutputFormat::Js | OutputFormat::Json => {
            fs::create_dir_all(&args.output).expect("Failed to create output directory");

            for (path, output) in results.into_iter().flatten() {
                let ext = match args.format {
                    OutputFormat::Js => get_output_extension(&output.script_lang, args.script_ext),
                    OutputFormat::Json => "json",
                    OutputFormat::Stats => unreachable!(),
                };

                let filename = path
                    .file_name()
                    .map(|f| PathBuf::from(f).with_extension(ext))
                    .unwrap_or_else(|| PathBuf::from("output").with_extension(ext));
                let out_path = args.output.join(filename);

                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).expect("Failed to create output subdirectory");
                }

                let content = match args.format {
                    OutputFormat::Js => output.code,
                    OutputFormat::Json => serde_json::to_string_pretty(&output).unwrap_or_default(),
                    OutputFormat::Stats => unreachable!(),
                };

                fs::write(&out_path, content).unwrap_or_else(|e| {
                    eprintln!("Failed to write {}: {}", out_path.display(), e);
                });
            }
        }
    }
    let io_elapsed = io_start.elapsed();

    let total_elapsed = start.elapsed();
    let success = stats.success.load(Ordering::Relaxed);
    let failed = stats.failed.load(Ordering::Relaxed);

    if args.profile {
        eprintln!();
        eprintln!("Timing breakdown:");
        eprintln!("  File collection: {:.4}s", collect_elapsed.as_secs_f64());
        eprintln!("  Compilation:     {:.4}s", compile_elapsed.as_secs_f64());
        eprintln!("  I/O operations:  {:.4}s", io_elapsed.as_secs_f64());
        eprintln!("  Total:           {:.4}s", total_elapsed.as_secs_f64());
        eprintln!();
    }

    if failed > 0 {
        eprintln!(
            "✗ {} file(s) failed, {} compiled in {:.4}s",
            failed,
            success,
            total_elapsed.as_secs_f64()
        );
    } else {
        let file_word = if success == 1 { "file" } else { "files" };
        eprintln!(
            "✓ {} {} compiled in {:.4}s",
            success,
            file_word,
            total_elapsed.as_secs_f64()
        );
    }

    if failed > 0 {
        std::process::exit(1);
    }
}

fn collect_files(patterns: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for pattern in patterns {
        let (root, glob_pattern) = parse_pattern(pattern);

        for entry in Walk::new(&root).flatten() {
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "vue")
                && pattern_matches(path, &glob_pattern)
            {
                files.push(path.to_path_buf());
            }
        }
    }

    files.sort();
    files.dedup();
    files
}

fn parse_pattern(pattern: &str) -> (String, String) {
    if let Some(pos) = pattern.find(['*', '?']) {
        let root_part = &pattern[..pos];
        if let Some(last_slash) = root_part.rfind('/') {
            let root = &pattern[..last_slash];
            let root = if root.is_empty() { "." } else { root };
            return (root.to_string(), pattern.to_string());
        }
    }

    let path = std::path::Path::new(pattern);
    if path.is_dir() {
        return (pattern.to_string(), format!("{}/**/*.vue", pattern));
    }

    if path.is_file() && pattern.ends_with(".vue") {
        if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            let parent_str = if parent_str.is_empty() {
                "."
            } else {
                &parent_str
            };
            return (parent_str.to_string(), pattern.to_string());
        }
    }

    (".".to_string(), pattern.to_string())
}

fn pattern_matches(path: &std::path::Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy().replace("\\", "/");

    if pattern == "./**/*.vue" || pattern == "**/*.vue" {
        return path_str.ends_with(".vue");
    }

    if pattern.contains("**/*.vue") {
        if let Some(prefix_end) = pattern.find("**") {
            let prefix = &pattern[..prefix_end];
            let prefix_normalized = prefix.trim_end_matches('/');
            return path_str.contains(&format!("{}/", prefix_normalized))
                && path_str.ends_with(".vue");
        }
    }

    if pattern.ends_with(".vue") {
        let pattern_normalized = pattern.replace("\\", "/");
        return path_str == pattern_normalized
            || path_str.ends_with(&format!("/{}", pattern_normalized));
    }

    path_str.ends_with(".vue")
}

fn detect_script_lang(source: &str) -> String {
    let script_pattern = regex_lite::Regex::new(r#"<script[^>]*\blang\s*=\s*["']([^"']+)["']"#)
        .expect("Invalid regex");

    if let Some(captures) = script_pattern.captures(source) {
        if let Some(lang) = captures.get(1) {
            return lang.as_str().to_string();
        }
    }

    "js".to_string()
}

fn compile_file(
    path: &PathBuf,
    ssr: bool,
    script_ext: ScriptExtension,
) -> Result<CompileOutput, String> {
    let source = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("anonymous.vue")
        .to_string();

    let script_lang = detect_script_lang(&source);

    let parse_opts = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };

    let descriptor = parse_sfc(&source, parse_opts).map_err(|e| e.message)?;

    let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
    let is_ts = matches!(script_ext, ScriptExtension::Preserve);
    let compile_opts = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            is_ts,
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.clone()),
            scoped: has_scoped,
            ssr,
            is_ts,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename.clone(),
            scoped: has_scoped,
            ..Default::default()
        },
    };

    let result = compile_sfc(&descriptor, compile_opts).map_err(|e| e.message)?;

    Ok(CompileOutput {
        filename,
        code: result.code,
        css: result.css,
        errors: result.errors.into_iter().map(|e| e.message).collect(),
        warnings: result.warnings.into_iter().map(|e| e.message).collect(),
        script_lang,
    })
}

fn get_output_extension(script_lang: &str, script_ext: ScriptExtension) -> &'static str {
    match script_ext {
        ScriptExtension::Downcompile => "js",
        ScriptExtension::Preserve => match script_lang {
            "ts" => "ts",
            "tsx" => "tsx",
            "jsx" => "jsx",
            _ => "js",
        },
    }
}
