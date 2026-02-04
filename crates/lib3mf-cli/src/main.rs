use clap::{Parser, Subcommand};
use lib3mf_cli::commands;
use lib3mf_cli::commands::{OutputFormat, RepairType};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "3mf")]
#[command(about = "A CLI tool for analyzing and processing 3MF files", long_about = None)]
#[cfg_attr(
    debug_assertions,
    command(version = concat!(
        env!("CARGO_PKG_VERSION"),
        "\ncommit: ", env!("VERGEN_GIT_SHA"),
        "\ndate:  ", env!("VERGEN_GIT_COMMIT_TIMESTAMP")
    ))
)]
#[cfg_attr(not(debug_assertions), command(version = env!("CARGO_PKG_VERSION")))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Report statistics and metadata for a 3MF file
    ///
    /// Analyzes the 3MF file and reports key metrics such as:
    ///
    /// * Unit of measurement
    ///
    /// * Geometry counts (vertices, triangles, objects)
    ///
    /// * Material group counts
    ///
    /// * Metadata entries
    ///
    /// Examples:
    ///
    /// # Generate a human-readable text report (default)
    ///
    /// $ lib3mf stats model.3mf
    ///
    /// # Generate a JSON report for machine parsing
    ///
    /// $ lib3mf stats model.3mf --format json
    Stats {
        /// Path to the 3MF file
        file: PathBuf,

        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Shortcut for --format tree
        #[arg(long, short)]
        tree: bool,
    },
    /// List all entries in the 3MF archive
    ///
    /// Displays a flat list or tree view of all files contained within the 3MF OPC archive.
    /// Useful for understanding the internal structure of the package.
    ///
    /// Examples:
    ///
    /// # List all files (flat view)
    ///
    /// $ lib3mf list model.3mf
    ///
    /// # Show directory structure as a tree
    ///
    /// $ lib3mf list model.3mf --format tree
    List {
        /// Path to the 3MF file
        file: PathBuf,

        /// Output format (text, json, tree)
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Shortcut for --format tree
        #[arg(long, short)]
        tree: bool,
    },
    /// Inspect OPC Relationships and Content Types
    ///
    /// Dumps the Open Packaging Convention (OPC) relationships and content types.
    ///
    /// Examples:
    ///
    /// # Show relationships and content types
    ///
    /// $ lib3mf rels model.3mf
    ///
    /// # Output as JSON
    ///
    /// $ lib3mf rels model.3mf --format json
    Rels {
        /// Path to the 3MF file
        file: PathBuf,

        /// Output format (text, json)
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Dump the raw parsed Model structure for debugging
    ///
    /// Detailed inspection of the in-memory representation of the 3MF model.
    /// This is primarily for developers debugging the parser.
    ///
    /// Examples:
    ///
    /// # Dump debug view to stdout
    ///
    /// $ lib3mf dump model.3mf
    Dump {
        /// Path to the 3MF file
        file: PathBuf,

        /// Output format (text, json)
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
    /// Extract a file from the archive
    ///
    /// Copies a specific file from inside the 3MF ZIP archive to the local filesystem.
    /// Use 'list' to see available files.
    ///
    /// Examples:
    ///
    /// # Extract the thumbnail image
    ///
    /// $ lib3mf extract model.3mf Metadata/thumbnail.png --output thumb.png
    ///
    /// # Extract content to stdout
    ///
    /// $ lib3mf extract model.3mf 3D/3dmodel.model
    ///
    /// # Extract displacement texture by resource ID
    ///
    /// $ lib3mf extract model.3mf --resource-id 100 --output height.png
    Extract {
        /// Path to the 3MF file
        file: PathBuf,

        /// Path to the file inside the archive to extract
        #[arg(conflicts_with = "resource_id")]
        inner_path: Option<String>,

        /// Resource ID of a texture to extract (Displacement2D or Texture2D)
        #[arg(long, conflicts_with = "inner_path")]
        resource_id: Option<u32>,

        /// Output path (defaults to stdout)
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Copy and re-package a 3MF file (verifies read/write cycle)
    ///
    /// Reads the input file into memory and writes it back to a new file.
    /// This effectively verifies that lib3mf can parse and re-serialize the model without errors.
    ///
    /// Examples:
    ///
    /// # Read, parse, and write back to a new file
    ///
    /// $ lib3mf copy source.3mf destination.3mf
    Copy {
        /// Input 3MF file
        input: PathBuf,
        /// Output 3MF file
        output: PathBuf,
    },
    /// Convert between 3D formats (3MF, STL, OBJ)
    ///
    /// Auto-detects the format based on file extensions.
    ///
    /// Supported Conversions:
    ///
    /// * STL (binary) -> 3MF
    ///
    /// * OBJ -> 3MF
    ///
    /// * 3MF -> STL (binary)
    ///
    /// * 3MF -> OBJ
    ///
    /// Examples:
    ///
    /// # Import STL to 3MF
    ///
    /// $ lib3mf convert mesh.stl model.3mf
    ///
    /// # Export 3MF to OBJ
    ///
    /// $ lib3mf convert model.3mf mesh.obj
    Convert {
        /// Input file
        input: PathBuf,
        /// Output file
        output: PathBuf,
    },
    /// Validate a 3MF file
    ///
    /// Performs semantic validation on the model to ensure it complies with the 3MF Core Specification.
    ///
    /// Levels:
    ///
    /// * minimal: Only check file integrity.
    ///
    /// * standard: Check for missing resources and basic structure.
    ///
    /// * strict: Enforce unit consistency and detailed schema rules.
    ///
    /// Examples:
    ///
    /// # Standard validation (default)
    ///
    /// $ lib3mf validate model.3mf
    ///
    /// # Strict validation
    ///
    /// $ lib3mf validate model.3mf --level strict
    Validate {
        /// Path to the 3MF file
        file: PathBuf,
        /// Validation level (minimal, standard, strict, paranoid)
        #[arg(long, default_value = "standard")]
        level: String,
    },
    /// Repair a 3MF mesh
    ///
    /// Performs advanced geometric processing to ensure 3D printability.
    /// Supports vertex stitching, degenerate triangle removal, orientation
    /// harmonization, hole filling, and disconnected component removal.
    ///
    /// Examples:
    ///
    /// # Repair a mesh
    ///
    /// $ lib3mf repair broken.3mf fixed.3mf
    Repair {
        /// Input file
        input: PathBuf,
        /// Output file
        output: PathBuf,

        /// Stitch epsilon for merging vertices
        #[arg(long, default_value = "1e-4")]
        epsilon: f32,

        /// Specific repairs to perform (degenerate, duplicates, harmonize, islands, holes, all)
        #[arg(
            long = "fix",
            short = 'f',
            value_delimiter = ',',
            default_value = "degenerate,duplicates,harmonize"
        )]
        fixes: Vec<RepairType>,
    },
    /// Sign a 3MF file using an RSA key
    ///
    /// Applies a digital signature to the 3MF package, ensuring authenticity and integrity.
    /// The signature works by hashing the model content and signing the hash with your private key.
    ///
    /// Examples:
    ///
    /// # Sign a 3MF file
    ///
    /// $ lib3mf sign input.3mf signed.3mf --key private.pem --cert public.crt
    Sign {
        /// Input 3MF file
        input: PathBuf,
        /// Output 3MF file
        output: PathBuf,
        /// Path to PEM-encoded private key
        #[arg(long)]
        key: PathBuf,
        /// Path to PEM-encoded certificate/public key
        #[arg(long)]
        cert: PathBuf,
    },
    /// Verify digital signatures in a 3MF file
    ///
    /// Checks all digital signatures present in the 3MF package.
    /// Reports whether signatures are valid, invalid, or missing.
    ///
    /// Examples:
    ///
    /// # Verify signatures
    ///
    /// $ lib3mf verify signed.3mf
    Verify {
        /// Path to the 3MF file
        file: PathBuf,
    },
    /// Encrypt a 3MF file to a recipient
    ///
    /// Encrypts the 3MF content for a specific recipient using their public certificate.
    /// Only the holder of the corresponding private key can decrypt the file.
    ///
    /// Examples:
    ///
    /// # Encrypt for a recipient
    ///
    /// $ lib3mf encrypt clear.3mf secret.3mf --recipient bob.crt
    Encrypt {
        /// Input file
        input: PathBuf,
        /// Output file
        output: PathBuf,
        /// Recipient certificate (PEM)
        #[arg(long)]
        recipient: PathBuf,
    },
    /// Decrypt a 3MF file
    ///
    /// Decrypts a secure 3MF file using your private key.
    ///
    /// Examples:
    ///
    /// # Decrypt a file
    ///
    /// $ lib3mf decrypt secret.3mf clear.3mf --key private.pem
    Decrypt {
        /// Input file
        input: PathBuf,
        /// Output file
        output: PathBuf,
        /// Private key (PEM)
        #[arg(long)]
        key: PathBuf,
    },
    /// Benchmark loading and parsing speed
    ///
    /// measures the time taken for various stages of the loading process:
    ///
    /// - ZIP Archive opening
    ///
    /// - XML Parsing
    ///
    /// - Statistics calculation
    ///
    /// Useful for performance profiling.
    ///
    /// Examples:
    ///
    /// # Benchmark a model
    ///
    /// $ lib3mf benchmark massive_model.3mf
    Benchmark {
        /// Path to the 3MF file
        file: PathBuf,
    },
    /// Compare two 3MF files
    ///
    /// Performs a structural comparison between two 3MF files.
    /// Detects differences in:
    ///
    /// - Metadata
    ///
    /// - Resource counts
    ///
    /// - Build item counts
    ///
    /// Examples:
    ///
    /// # Diff two files
    ///
    /// $ lib3mf diff v1.3mf v2.3mf
    Diff {
        /// First file
        file1: PathBuf,
        /// Second file
        file2: PathBuf,
        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Manage thumbnails (extract, inject, list)
    ///
    /// Examples:
    ///
    /// # List all thumbnails and OIDs
    /// $ lib3mf thumbnails input.3mf --list
    ///
    /// # Inject package thumbnail
    /// $ lib3mf thumbnails input.3mf --inject thumb.png
    ///
    /// # Inject object thumbnail
    /// $ lib3mf thumbnails input.3mf --inject thumb.png --oid 1
    Thumbnails {
        /// Input 3MF file
        file: PathBuf,
        /// List all thumbnails and OIDs
        #[arg(long)]
        list: bool,
        /// Extract all thumbnails to directory
        #[arg(long)]
        extract: Option<PathBuf>,
        /// Inject image file
        #[arg(long)]
        inject: Option<PathBuf>,
        /// Target Object ID (for injection)
        #[arg(long)]
        oid: Option<u32>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Stats { file, format, tree } => {
            let format = if tree { OutputFormat::Tree } else { format };
            commands::stats(file, format)?;
        }
        Commands::List { file, format, tree } => {
            let format = if tree { OutputFormat::Tree } else { format };
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
            resource_id,
            output,
        } => {
            if let Some(rid) = resource_id {
                commands::extract_by_resource_id(file, rid, output)?;
            } else if let Some(path) = inner_path {
                commands::extract(file, path, output)?;
            } else {
                anyhow::bail!("Either inner_path or --resource-id must be provided");
            }
        }
        Commands::Copy { input, output } => {
            commands::copy(input, output)?;
        }
        Commands::Convert { input, output } => {
            commands::convert(input, output)?;
        }
        Commands::Validate { file, level } => {
            commands::validate(file, level)?;
        }
        Commands::Repair {
            input,
            output,
            epsilon,
            fixes,
        } => {
            commands::repair(input, output, epsilon, fixes)?;
        }
        Commands::Sign {
            input,
            output,
            key,
            cert,
        } => {
            commands::sign(input, output, key, cert)?;
        }
        Commands::Verify { file } => {
            commands::verify(file)?;
        }
        Commands::Encrypt {
            input,
            output,
            recipient,
        } => {
            commands::encrypt(input, output, recipient)?;
        }
        Commands::Decrypt { input, output, key } => {
            commands::decrypt(input, output, key)?;
        }
        Commands::Benchmark { file } => {
            commands::benchmark(file)?;
        }
        Commands::Diff {
            file1,
            file2,
            format,
        } => {
            commands::diff(file1, file2, &format)?;
        }
        Commands::Thumbnails {
            file,
            list,
            extract,
            inject,
            oid,
        } => {
            commands::thumbnails::run(file, list, extract, inject, oid)?;
        }
    }

    Ok(())
}
