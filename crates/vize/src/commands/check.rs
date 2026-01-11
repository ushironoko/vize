//! Check command - Type check Vue SFC files

use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct CheckArgs {
    /// Glob pattern(s) to match .vue files
    #[arg(default_value = "./**/*.vue")]
    pub patterns: Vec<String>,

    /// tsconfig.json path
    #[arg(long)]
    pub tsconfig: Option<PathBuf>,

    /// Strict mode
    #[arg(long)]
    pub strict: bool,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

pub fn run(args: CheckArgs) {
    eprintln!("vize check: Type checking Vue SFC files...");
    eprintln!("  patterns: {:?}", args.patterns);
    eprintln!("  strict: {}", args.strict);
    eprintln!("  format: {}", args.format);

    // TODO: Implement type checking
    eprintln!("Type checking is not yet implemented.");
}
