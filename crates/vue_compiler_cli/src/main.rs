//! Vue Compiler CLI
//!
//! A high-performance CLI for compiling Vue SFC files with native multithreading.

use clap::{Parser, ValueEnum};
use glob::glob;
use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser as OxcParser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use vue_compiler_sfc::{
    compile_sfc, parse_sfc, ScriptCompileOptions, SfcCompileOptions, SfcParseOptions,
    StyleCompileOptions, TemplateCompileOptions,
};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Output compiled JavaScript
    Js,
    /// Output JSON with code and metadata
    Json,
    /// Only show statistics (no output)
    Stats,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum ScriptExtension {
    /// Preserve original script language extension (.ts -> .ts, .tsx -> .tsx, .jsx -> .jsx)
    #[default]
    Preserve,
    /// Downcompile all scripts to JavaScript (.ts -> .js, .tsx -> .js, .jsx -> .js)
    Downcompile,
}

#[derive(Parser)]
#[command(name = "vue-compiler")]
#[command(about = "High-performance Vue SFC compiler", long_about = None)]
struct Cli {
    /// Glob pattern(s) to match .vue files (default: ./**/*.vue)
    #[arg(default_value = "./**/*.vue")]
    patterns: Vec<String>,

    /// Output directory (default: ./dist)
    #[arg(short, long, default_value = "./dist")]
    output: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value = "js")]
    format: OutputFormat,

    /// Enable SSR mode
    #[arg(long)]
    ssr: bool,

    /// Script extension handling: 'preserve' keeps original extension (.ts/.tsx/.jsx), 'downcompile' converts to .js
    #[arg(long, value_enum, default_value = "preserve")]
    script_ext: ScriptExtension,

    /// Number of threads (default: number of CPUs)
    #[arg(short = 'j', long)]
    threads: Option<usize>,

    /// Show timing profile breakdown
    #[arg(long)]
    profile: bool,

    /// Continue on errors
    #[arg(long)]
    continue_on_error: bool,
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
    /// The detected script language (ts, tsx, jsx, or js)
    script_lang: String,
}

fn collect_files(patterns: &[String]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for pattern in patterns {
        match glob(pattern) {
            Ok(paths) => {
                for entry in paths.flatten() {
                    if entry.extension().is_some_and(|ext| ext == "vue") {
                        files.push(entry);
                    }
                }
            }
            Err(e) => {
                eprintln!("Invalid glob pattern '{}': {}", pattern, e);
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

/// Detect the script language from Vue SFC source
fn detect_script_lang(source: &str) -> String {
    // Look for <script setup lang="..."> or <script lang="...">
    let script_pattern = regex_lite::Regex::new(r#"<script[^>]*\blang\s*=\s*["']([^"']+)["']"#)
        .expect("Invalid regex");

    if let Some(captures) = script_pattern.captures(source) {
        if let Some(lang) = captures.get(1) {
            return lang.as_str().to_string();
        }
    }

    // Default to js if no lang attribute found
    "js".to_string()
}

/// Transpile TypeScript/TSX/JSX to JavaScript using oxc
fn transpile_to_js(code: &str, filename: &str, lang: &str) -> Result<String, String> {
    let source_type = match lang {
        "ts" => SourceType::ts(),
        "tsx" => SourceType::tsx(),
        "jsx" => SourceType::jsx(),
        _ => return Ok(code.to_string()), // Already JS
    };

    let allocator = Allocator::default();
    let ret = OxcParser::new(&allocator, code, source_type).parse();

    if !ret.errors.is_empty() {
        let error_messages: Vec<_> = ret.errors.iter().map(|e| e.to_string()).collect();
        return Err(format!("Parse errors: {}", error_messages.join(", ")));
    }

    let mut program = ret.program;

    // Run semantic analysis to get symbols and scopes
    let semantic_ret = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .build(&program);

    if !semantic_ret.errors.is_empty() {
        // If semantic analysis fails, return original code
        return Ok(code.to_string());
    }

    let (symbols, scopes) = semantic_ret.semantic.into_symbol_table_and_scope_tree();

    // Transform TypeScript/JSX to JavaScript
    let transform_options = TransformOptions::default();
    let ret = Transformer::new(&allocator, Path::new(filename), &transform_options)
        .build_with_symbols_and_scopes(symbols, scopes, &mut program);

    if !ret.errors.is_empty() {
        // If transformation fails, return original code
        return Ok(code.to_string());
    }

    // Generate JavaScript code
    let codegen = Codegen::new();
    let result = codegen.build(&program);

    Ok(result.code)
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

    // Detect script language
    let script_lang = detect_script_lang(&source);

    // Parse
    let parse_opts = SfcParseOptions {
        filename: filename.clone(),
        ..Default::default()
    };

    let descriptor = parse_sfc(&source, parse_opts).map_err(|e| e.message)?;

    // Compile
    let has_scoped = descriptor.styles.iter().any(|s| s.scoped);
    let compile_opts = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: filename.clone(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(filename.clone()),
            ..Default::default()
        },
        template: TemplateCompileOptions {
            id: Some(filename.clone()),
            scoped: has_scoped,
            ssr,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: filename.clone(),
            scoped: has_scoped,
            ..Default::default()
        },
    };

    let result = compile_sfc(&descriptor, compile_opts).map_err(|e| e.message)?;

    // Transpile to JS if downcompile mode is enabled and the script is TypeScript/JSX
    let output_code = match script_ext {
        ScriptExtension::Downcompile if script_lang != "js" => {
            transpile_to_js(&result.code, &filename, &script_lang)?
        }
        _ => result.code,
    };

    Ok(CompileOutput {
        filename,
        code: output_code,
        css: result.css,
        errors: result.errors.into_iter().map(|e| e.message).collect(),
        warnings: result.warnings.into_iter().map(|e| e.message).collect(),
        script_lang,
    })
}

/// Get the output file extension based on script language and extension mode
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

fn main() {
    let cli = Cli::parse();

    // Start timer (includes file collection time)
    let start = Instant::now();

    // Configure thread pool
    if let Some(threads) = cli.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .expect("Failed to configure thread pool");
    }

    // Collect files
    let files = collect_files(&cli.patterns);

    if files.is_empty() {
        eprintln!("No .vue files found matching the patterns");
        std::process::exit(1);
    }

    let stats = CompileStats::new(files.len());
    let collect_elapsed = start.elapsed();

    if cli.profile {
        eprintln!(
            "Found {} files in {:.2}s. Compiling using {} threads...",
            files.len(),
            collect_elapsed.as_secs_f64(),
            rayon::current_num_threads()
        );
    }

    // Parallel compilation
    let compile_start = Instant::now();
    let results: Vec<_> = files
        .par_iter()
        .map(|path| {
            let source_size = fs::metadata(path).map(|m| m.len() as usize).unwrap_or(0);
            stats.total_bytes.fetch_add(source_size, Ordering::Relaxed);

            match compile_file(path, cli.ssr, cli.script_ext) {
                Ok(output) => {
                    stats.success.fetch_add(1, Ordering::Relaxed);
                    stats
                        .output_bytes
                        .fetch_add(output.code.len(), Ordering::Relaxed);

                    if cli.profile && !output.errors.is_empty() {
                        for err in &output.errors {
                            eprintln!("  {} warning: {}", path.display(), err);
                        }
                    }

                    Some((path.clone(), output))
                }
                Err(e) => {
                    stats.failed.fetch_add(1, Ordering::Relaxed);
                    eprintln!("Error compiling {}: {}", path.display(), e);

                    if !cli.continue_on_error {
                        std::process::exit(1);
                    }

                    None
                }
            }
        })
        .collect();
    let compile_elapsed = compile_start.elapsed();

    // Output results
    let io_start = Instant::now();
    match cli.format {
        OutputFormat::Stats => {
            // Just show stats, handled below
        }
        OutputFormat::Js | OutputFormat::Json => {
            // Create output directory
            fs::create_dir_all(&cli.output).expect("Failed to create output directory");

            for (path, output) in results.into_iter().flatten() {
                let ext = match cli.format {
                    OutputFormat::Js => get_output_extension(&output.script_lang, cli.script_ext),
                    OutputFormat::Json => "json",
                    OutputFormat::Stats => unreachable!(),
                };

                // Use just the filename to avoid absolute path issues
                let filename = path
                    .file_name()
                    .map(|f| PathBuf::from(f).with_extension(ext))
                    .unwrap_or_else(|| PathBuf::from("output").with_extension(ext));
                let out_path = cli.output.join(filename);

                // Create parent directories if needed
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).expect("Failed to create output subdirectory");
                }

                let content = match cli.format {
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

    // Measure total elapsed time (including I/O)
    let total_elapsed = start.elapsed();

    // Print stats
    let success = stats.success.load(Ordering::Relaxed);
    let failed = stats.failed.load(Ordering::Relaxed);

    if cli.profile {
        eprintln!();
        eprintln!("Timing breakdown:");
        eprintln!("  File collection: {:.2}s", collect_elapsed.as_secs_f64());
        eprintln!("  Compilation:     {:.2}s", compile_elapsed.as_secs_f64());
        eprintln!("  I/O operations:  {:.2}s", io_elapsed.as_secs_f64());
        eprintln!("  Total:           {:.2}s", total_elapsed.as_secs_f64());
        eprintln!();
    }

    if failed > 0 {
        eprintln!(
            "✗ {} file(s) failed, {} compiled in {:.2}s",
            failed,
            success,
            total_elapsed.as_secs_f64()
        );
    } else {
        let file_word = if success == 1 { "file" } else { "files" };
        eprintln!(
            "✓ {} {} compiled in {:.2}s",
            success,
            file_word,
            total_elapsed.as_secs_f64()
        );
    }

    if failed > 0 {
        std::process::exit(1);
    }
}
