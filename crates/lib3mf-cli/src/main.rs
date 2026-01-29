mod commands;
use clap::{Parser, Subcommand};
use commands::OutputFormat;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "3mf")]
#[command(about = "A CLI tool for analyzing and processing 3MF files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Report statistics and metadata for a 3MF file
    Stats {
        /// Path to the 3MF file
        file: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// List all entries in the 3MF archive
    List {
        /// Path to the 3MF file
        file: PathBuf,

        /// Output format (text, json, tree)
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Inspect OPC Relationships and Content Types
    Rels {
        /// Path to the 3MF file
        file: PathBuf,

        /// Output format (text, json)
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Dump the raw parsed Model structure for debugging
    Dump {
        /// Path to the 3MF file
        file: PathBuf,

        /// Output format (text, json)
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Extract a file from the archive
    Extract {
        /// Path to the 3MF file
        file: PathBuf,

        /// Path to the file inside the archive to extract
        inner_path: String,

        /// Output path (defaults to stdout)
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Copy and re-package a 3MF file (verifies read/write cycle)
    Copy {
        /// Input 3MF file
        input: PathBuf,
        /// Output 3MF file
        output: PathBuf,
    },
    /// Convert between 3D formats (3MF, STL, OBJ)
    Convert {
        /// Input file
        input: PathBuf,
        /// Output file
        output: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Stats { file, format } => {
            commands::stats(file, format)?;
        }
        Commands::List { file, format } => {
            commands::list(file, format)?;
        }
        Commands::Rels { file, format } => {
            commands::rels(file, format)?;
        }
        Commands::Dump { file, format } => {
            commands::dump(file, format)?;
        }
        Commands::Extract {
            file,
            inner_path,
            output,
        } => {
            commands::extract(file, inner_path, output)?;
        }
        Commands::Copy { input, output } => {
            commands::copy(input, output)?;
        }
        Commands::Convert { input, output } => {
            commands::convert(input, output)?;
        }
    }

    Ok(())
}
