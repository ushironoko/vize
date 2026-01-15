//! Check command - Type check Vue SFC files
//!
//! Generates Virtual TypeScript from Vue SFCs and uses tsgo LSP for type checking.
//! Can connect to a running check-server via Unix socket for faster repeated checks.

use clap::Args;
use glob::glob;
use ignore::Walk;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Args)]
pub struct CheckArgs {
    /// Glob pattern(s) to match .vue files
    #[arg(default_value = "./**/*.vue")]
    pub patterns: Vec<String>,

    /// Connect to check-server via Unix socket (faster for repeated checks)
    #[arg(long, short)]
    pub socket: Option<String>,

    /// tsconfig.json path
    #[arg(long)]
    pub tsconfig: Option<PathBuf>,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Show generated virtual TypeScript
    #[arg(long)]
    pub show_virtual_ts: bool,

    /// Quiet mode - only show summary
    #[arg(short, long)]
    pub quiet: bool,
}

/// JSON output structure
#[derive(Serialize)]
struct JsonOutput {
    files: Vec<JsonFileResult>,
    #[serde(rename = "errorCount")]
    error_count: usize,
    #[serde(rename = "fileCount")]
    file_count: usize,
}

#[derive(Serialize)]
struct JsonFileResult {
    file: String,
    #[serde(rename = "virtualTs")]
    virtual_ts: String,
    diagnostics: Vec<String>,
}

struct GeneratedFile {
    original: String,
    virtual_ts: String,
}

/// Server response for check method
#[derive(Deserialize)]
struct ServerCheckResult {
    diagnostics: Vec<ServerDiagnostic>,
    #[serde(rename = "virtualTs")]
    virtual_ts: String,
    #[serde(rename = "errorCount")]
    error_count: usize,
}

#[derive(Deserialize)]
struct ServerDiagnostic {
    message: String,
    severity: String,
    line: u32,
    column: u32,
    code: Option<String>,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<ServerCheckResult>,
    error: Option<JsonRpcError>,
}

#[derive(Deserialize)]
struct JsonRpcError {
    #[allow(dead_code)]
    code: i64,
    message: String,
}

pub fn run(args: CheckArgs) {
    // If socket is specified, use socket client mode
    if let Some(ref socket_path) = args.socket {
        run_with_socket(&args, socket_path);
        return;
    }

    // Otherwise, fall back to direct tsgo execution
    run_direct(&args);
}

/// Run type checking via Unix socket connection to check-server
fn run_with_socket(args: &CheckArgs, socket_path: &str) {
    let start = Instant::now();

    // Collect files
    let files: Vec<PathBuf> = collect_vue_files(&args.patterns);

    if files.is_empty() {
        eprintln!("No .vue files found matching patterns: {:?}", args.patterns);
        return;
    }

    // Connect to server
    let mut stream = match UnixStream::connect(socket_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "\x1b[31mError:\x1b[0m Failed to connect to check-server: {}",
                e
            );
            eprintln!();
            eprintln!("\x1b[33mHint:\x1b[0m Start the server first:");
            eprintln!("  vize check-server --socket {}", socket_path);
            std::process::exit(1);
        }
    };

    if !args.quiet {
        eprintln!("Connected to check-server at {}", socket_path);
        eprintln!("Type checking {} files...", files.len());
    }

    let mut total_errors = 0;
    let mut results: Vec<(String, ServerCheckResult)> = Vec::new();

    for path in &files {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to read {}: {}", path.display(), e);
                continue;
            }
        };

        let filename = path.to_string_lossy().to_string();

        // Send request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "check",
            "params": {
                "uri": filename,
                "content": source
            }
        });

        if writeln!(stream, "{}", request).is_err() {
            eprintln!("Failed to send request");
            break;
        }
        if stream.flush().is_err() {
            eprintln!("Failed to flush");
            break;
        }

        // Read response
        let mut reader = BufReader::new(&stream);
        let mut response_line = String::new();
        if reader.read_line(&mut response_line).is_err() {
            eprintln!("Failed to read response");
            break;
        }

        let response: JsonRpcResponse = match serde_json::from_str(&response_line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to parse response: {}", e);
                continue;
            }
        };

        if let Some(error) = response.error {
            eprintln!("Server error: {}", error.message);
            continue;
        }

        if let Some(result) = response.result {
            total_errors += result.error_count;

            if args.show_virtual_ts {
                eprintln!("\n=== {} ===", filename);
                eprintln!("{}", result.virtual_ts);
            }

            results.push((filename, result));
        }
    }

    let total_time = start.elapsed();

    // Print results
    if !args.quiet {
        for (filename, result) in &results {
            if result.diagnostics.is_empty() {
                continue;
            }

            println!("\n\x1b[4m{}\x1b[0m", filename);
            for diag in &result.diagnostics {
                let color = if diag.severity == "error" {
                    "\x1b[31m"
                } else {
                    "\x1b[33m"
                };
                let code_str = diag
                    .code
                    .as_ref()
                    .map(|c| format!(" [{}]", c))
                    .unwrap_or_default();
                println!(
                    "  {}{}:{}:{}\x1b[0m{} {}",
                    color, diag.severity, diag.line, diag.column, code_str, diag.message
                );
            }
        }
    }

    // Print summary
    let status = if total_errors > 0 {
        "\x1b[31m✗\x1b[0m"
    } else {
        "\x1b[32m✓\x1b[0m"
    };
    println!(
        "\n{} Type checked {} files in {:.2?} (via socket)",
        status,
        files.len(),
        total_time
    );

    if total_errors > 0 {
        println!("  \x1b[31m{} error(s)\x1b[0m", total_errors);
        std::process::exit(1);
    } else {
        println!("  \x1b[32mNo type errors found!\x1b[0m");
    }
}

/// Collect .vue files from patterns
fn collect_vue_files(patterns: &[String]) -> Vec<PathBuf> {
    patterns
        .iter()
        .flat_map(|pattern| {
            if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                glob(pattern)
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|r| r.ok())
                    .filter(|p| {
                        p.extension().is_some_and(|ext| ext == "vue")
                            && !p.components().any(|c| c.as_os_str() == "node_modules")
                    })
                    .collect::<Vec<_>>()
            } else {
                Walk::new(pattern)
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "vue"))
                    .map(|e| e.path().to_path_buf())
                    .collect::<Vec<_>>()
            }
        })
        .collect()
}

/// Run type checking directly with tsgo LSP (no file I/O)
fn run_direct(args: &CheckArgs) {
    use vize_atelier_core::parser::parse;
    use vize_atelier_sfc::{parse_sfc, SfcParseOptions};
    use vize_canon::lsp_client::TsgoLspClient;
    use vize_carton::Bump;
    use vize_croquis::virtual_ts::generate_virtual_ts;
    use vize_croquis::{Analyzer, AnalyzerOptions};

    let start = Instant::now();

    // Collect .vue files
    let files = collect_vue_files(&args.patterns);

    if files.is_empty() {
        eprintln!("No .vue files found matching patterns: {:?}", args.patterns);
        return;
    }

    if !args.quiet {
        eprintln!("Generating Virtual TypeScript for {} files...", files.len());
    }

    let gen_start = Instant::now();

    // Generate Virtual TypeScript for each file
    let generated: Vec<GeneratedFile> = files
        .iter()
        .filter_map(|path| {
            let source = fs::read_to_string(path).ok()?;
            // Use absolute path for proper file:// URI
            let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            let filename = abs_path.to_string_lossy().to_string();

            // Parse SFC
            let parse_opts = SfcParseOptions {
                filename: filename.clone(),
                ..Default::default()
            };
            let descriptor = parse_sfc(&source, parse_opts).ok()?;

            // Get script content
            let script_content = descriptor
                .script_setup
                .as_ref()
                .map(|s| s.content.as_ref())
                .or_else(|| descriptor.script.as_ref().map(|s| s.content.as_ref()));

            // Create allocator
            let allocator = Bump::new();

            // Analyze
            let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());

            let template_offset: u32 = if let Some(ref script_setup) = descriptor.script_setup {
                analyzer.analyze_script_setup(&script_setup.content);
                descriptor
                    .template
                    .as_ref()
                    .map(|t| t.loc.start as u32)
                    .unwrap_or(0)
            } else if let Some(ref script) = descriptor.script {
                analyzer.analyze_script_plain(&script.content);
                descriptor
                    .template
                    .as_ref()
                    .map(|t| t.loc.start as u32)
                    .unwrap_or(0)
            } else {
                0
            };

            let template_ast = if let Some(ref template) = descriptor.template {
                let (root, _) = parse(&allocator, &template.content);
                analyzer.analyze_template(&root);
                Some(root)
            } else {
                None
            };

            let summary = analyzer.finish();

            // Generate Virtual TS
            let output = generate_virtual_ts(
                script_content,
                template_ast.as_ref(),
                &summary.bindings,
                None,
                Some(std::path::Path::new(&filename)),
                template_offset,
            );

            Some(GeneratedFile {
                original: filename,
                virtual_ts: output.content,
            })
        })
        .collect();

    let gen_time = gen_start.elapsed();

    if generated.is_empty() {
        eprintln!("No files to check");
        return;
    }

    if args.show_virtual_ts {
        for g in &generated {
            eprintln!("\n=== {} ===", g.original);
            eprintln!("{}", g.virtual_ts);
        }
    }

    if !args.quiet {
        eprintln!("Running tsgo LSP on {} files...", generated.len());
    }

    let check_start = Instant::now();

    // Initialize LSP client
    let mut lsp_client = match TsgoLspClient::new(None, None) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("\x1b[31mError:\x1b[0m Failed to start tsgo LSP: {}", e);
            eprintln!();
            eprintln!("\x1b[33mHint:\x1b[0m Install tsgo:");
            eprintln!("  npm install -g @anthropic/native-preview  # tsgo");
            std::process::exit(1);
        }
    };

    let mut total_errors = 0;
    let mut all_diagnostics: Vec<(String, Vec<String>)> = Vec::new();

    // Check each file via LSP
    for g in &generated {
        let virtual_uri = format!("file://{}.ts", g.original);

        // Send virtual file to LSP
        if let Err(e) = lsp_client.did_open(&virtual_uri, &g.virtual_ts) {
            eprintln!("Failed to open {}: {}", g.original, e);
            continue;
        }

        // Get diagnostics
        let diagnostics = lsp_client.get_diagnostics(&virtual_uri);

        // Close virtual file
        let _ = lsp_client.did_close(&virtual_uri);

        // Collect diagnostics
        let mut file_diags: Vec<String> = Vec::new();
        for diag in &diagnostics {
            let severity = match diag.severity {
                Some(1) => {
                    total_errors += 1;
                    "error"
                }
                Some(2) => "warning",
                _ => "error",
            };
            let code_str = diag
                .code
                .as_ref()
                .map(|c| match c {
                    serde_json::Value::Number(n) => format!(" [TS{}]", n),
                    serde_json::Value::String(s) => format!(" [{}]", s),
                    _ => String::new(),
                })
                .unwrap_or_default();
            let line = diag.range.start.line + 1;
            let col = diag.range.start.character + 1;
            file_diags.push(format!(
                "{}:{}:{}{} {}",
                severity, line, col, code_str, diag.message
            ));
        }

        if !file_diags.is_empty() {
            all_diagnostics.push((g.original.clone(), file_diags));
        }
    }

    let check_time = check_start.elapsed();
    let total_time = start.elapsed();

    // JSON output
    if args.format == "json" {
        let json_output = JsonOutput {
            files: generated
                .iter()
                .map(|g| {
                    let diags = all_diagnostics
                        .iter()
                        .find(|(f, _)| f == &g.original)
                        .map(|(_, d)| d.clone())
                        .unwrap_or_default();
                    JsonFileResult {
                        file: g.original.clone(),
                        virtual_ts: g.virtual_ts.clone(),
                        diagnostics: diags,
                    }
                })
                .collect(),
            error_count: total_errors,
            file_count: generated.len(),
        };
        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        return;
    }

    // Print diagnostics
    if !args.quiet {
        for (filename, diags) in &all_diagnostics {
            println!("\n\x1b[4m{}\x1b[0m", filename);
            for diag in diags {
                let color = if diag.starts_with("error") {
                    "\x1b[31m"
                } else {
                    "\x1b[33m"
                };
                println!("  {}{}\x1b[0m", color, diag);
            }
        }
    }

    // Print summary
    let status = if total_errors > 0 {
        "\x1b[31m✗\x1b[0m"
    } else {
        "\x1b[32m✓\x1b[0m"
    };

    println!(
        "\n{} Type checked {} files in {:.2?} (gen: {:.2?}, lsp: {:.2?})",
        status,
        generated.len(),
        total_time,
        gen_time,
        check_time
    );

    if total_errors > 0 {
        println!("  \x1b[31m{} error(s)\x1b[0m", total_errors);
        std::process::exit(1);
    } else {
        println!("  \x1b[32mNo type errors found!\x1b[0m");
    }
}
