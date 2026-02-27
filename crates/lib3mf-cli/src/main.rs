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
    /// * STL (binary/ASCII) -> 3MF
    ///
    /// * OBJ -> 3MF
    ///
    /// * 3MF -> STL (binary, or ASCII with --ascii)
    ///
    /// * 3MF -> OBJ
    ///
    /// Examples:
    ///
    /// # Import STL to 3MF
    ///
    /// $ lib3mf convert mesh.stl model.3mf
    ///
    /// # Export 3MF to ASCII STL
    ///
    /// $ lib3mf convert model.3mf mesh.stl --ascii
    ///
    /// # Export 3MF to OBJ
    ///
    /// $ lib3mf convert model.3mf mesh.obj
    Convert {
        /// Input file
        input: PathBuf,
        /// Output file
        output: PathBuf,
        /// Write ASCII STL instead of binary (only applies when output is .stl)
        #[arg(long, default_value_t = false)]
        ascii: bool,
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
    /// Merge multiple 3MF files into one
    ///
    /// Combines two or more 3MF files into a single output file.
    /// Resource IDs are remapped to avoid collisions.
    /// Secure (signed/encrypted) files cannot be merged.
    ///
    /// Examples:
    ///
    /// # Merge two files (default: plate-per-file, transforms preserved)
    ///
    /// $ lib3mf merge a.3mf b.3mf -o merged.3mf
    ///
    /// # Merge all files in a directory
    ///
    /// $ lib3mf merge "models/*.3mf" -o merged.3mf
    ///
    /// # Merge onto a single plate with grid layout
    ///
    /// $ lib3mf merge a.3mf b.3mf --single-plate -o merged.3mf
    Merge {
        /// Input 3MF files or glob patterns (e.g., "models/*.3mf")
        #[arg(required = true, num_args = 1..)]
        inputs: Vec<PathBuf>,

        /// Output file path
        #[arg(long, short = 'o', default_value = "merged.3mf")]
        output: PathBuf,

        /// Overwrite output file if it already exists
        #[arg(long, short = 'f')]
        force: bool,

        /// Merge all objects onto a single plate with auto-arrangement
        #[arg(long, conflicts_with = "plate_per_file")]
        single_plate: bool,

        /// Keep each file's objects on their own plate (default)
        #[arg(long, conflicts_with = "single_plate")]
        plate_per_file: bool,

        /// Packing algorithm for --single-plate mode
        #[arg(long, value_enum, default_value_t = commands::merge::PackAlgorithm::Grid)]
        pack: commands::merge::PackAlgorithm,

        /// Suppress all output
        #[arg(long, conflicts_with = "verbose")]
        quiet: bool,

        /// Show per-file progress and renumbering details
        #[arg(long, short = 'v', conflicts_with = "quiet")]
        verbose: bool,
    },
    /// Split a 3MF file into separate files per object or build item
    ///
    /// Extracts individual objects or plates from a 3MF file into separate
    /// output files. Each output contains only the resources needed by its
    /// object, with compact renumbered IDs.
    ///
    /// Examples:
    ///
    /// # Split by build item (default)
    ///
    /// $ lib3mf split model.3mf
    ///
    /// # Split by object resource
    ///
    /// $ lib3mf split model.3mf --by-object
    ///
    /// # Cherry-pick specific items
    ///
    /// $ lib3mf split model.3mf --select 0,Gear
    ///
    /// # Preview without writing
    ///
    /// $ lib3mf split model.3mf --dry-run
    Split {
        /// Input 3MF file to split
        input: PathBuf,

        /// Output directory (default: <input_stem>_split/ next to input)
        #[arg(long, short = 'o')]
        output_dir: Option<PathBuf>,

        /// Split by build item (default)
        #[arg(long, conflicts_with = "by_object")]
        by_item: bool,

        /// Split by unique object resource
        #[arg(long, conflicts_with = "by_item")]
        by_object: bool,

        /// Select specific items by index or name (comma-separated)
        #[arg(long, value_delimiter = ',')]
        select: Vec<String>,

        /// Keep original position/rotation/scale from source
        #[arg(long)]
        preserve_transforms: bool,

        /// Preview output files without writing
        #[arg(long)]
        dry_run: bool,

        /// Overwrite existing output directory/files
        #[arg(long, short = 'f')]
        force: bool,

        /// Suppress all output
        #[arg(long, conflicts_with = "verbose")]
        quiet: bool,

        /// Show per-item dependency tracing details
        #[arg(long, short = 'v', conflicts_with = "quiet")]
        verbose: bool,
    },
    /// Process multiple 3MF/STL/OBJ files with batch operations
    ///
    /// Runs one or more operations (validate, stats, list, convert) across
    /// multiple files in a single invocation. Files can be specified as paths,
    /// directories, or glob patterns.
    ///
    /// Examples:
    ///
    /// # Validate all 3MF files in a directory
    ///
    /// $ lib3mf batch ./models/ --validate
    ///
    /// # Validate and compute stats, JSON Lines output
    ///
    /// $ lib3mf batch "*.3mf" --validate --stats --format json
    ///
    /// # Convert all 3MF files to STL in parallel
    ///
    /// $ lib3mf batch ./models/ --convert --jobs 4
    ///
    /// # Recursive batch validate with summary
    ///
    /// $ lib3mf batch ./models/ --validate --recursive --summary
    Batch {
        /// Input paths, directories, or glob patterns
        #[arg(required = true, num_args = 1..)]
        inputs: Vec<PathBuf>,

        /// Validate each file
        #[arg(long)]
        validate: bool,

        /// Validation level (minimal, standard, strict, paranoid)
        #[arg(long, default_value = "standard")]
        validate_level: String,

        /// Report statistics for each file
        #[arg(long)]
        stats: bool,

        /// List archive entries for each file
        #[arg(long)]
        list: bool,

        /// Convert files (3MF to STL, or STL/OBJ to 3MF)
        #[arg(long)]
        convert: bool,

        /// Write ASCII STL instead of binary (only for --convert with 3MF->STL)
        #[arg(long, default_value_t = false)]
        convert_ascii: bool,

        /// Output directory for --convert (default: next to source file)
        #[arg(long)]
        output_dir: Option<PathBuf>,

        /// Number of parallel jobs (default: 1 = sequential)
        #[arg(long, short = 'j', default_value = "1")]
        jobs: usize,

        /// Recurse into subdirectories
        #[arg(long, short = 'r')]
        recursive: bool,

        /// Add summary table at end
        #[arg(long)]
        summary: bool,

        /// Output format (text, json)
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Suppress large-file-count warning
        #[arg(long)]
        yes: bool,

        /// Suppress all output
        #[arg(long, conflicts_with = "verbose")]
        quiet: bool,

        /// Verbose per-file output
        #[arg(long, short = 'v', conflicts_with = "quiet")]
        verbose: bool,
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
        Commands::Convert {
            input,
            output,
            ascii,
        } => {
            commands::convert(input, output, ascii)?;
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
        Commands::Merge {
            inputs,
            output,
            force,
            single_plate,
            plate_per_file: _,
            pack,
            quiet,
            verbose,
        } => {
            let verbosity = if quiet {
                commands::merge::Verbosity::Quiet
            } else if verbose {
                commands::merge::Verbosity::Verbose
            } else {
                commands::merge::Verbosity::Normal
            };
            commands::merge::run(inputs, output, force, single_plate, pack, verbosity)?;
        }
        Commands::Split {
            input,
            output_dir,
            by_item: _,
            by_object,
            select,
            preserve_transforms,
            dry_run,
            force,
            quiet,
            verbose,
        } => {
            let mode = if by_object {
                commands::split::SplitMode::ByObject
            } else {
                commands::split::SplitMode::ByItem
            };
            let verbosity = if quiet {
                commands::merge::Verbosity::Quiet
            } else if verbose {
                commands::merge::Verbosity::Verbose
            } else {
                commands::merge::Verbosity::Normal
            };
            commands::split::run(
                input,
                output_dir,
                mode,
                select,
                preserve_transforms,
                dry_run,
                force,
                verbosity,
            )?;
        }
        Commands::Batch {
            inputs,
            validate,
            validate_level,
            stats,
            list,
            convert,
            convert_ascii,
            output_dir,
            jobs,
            recursive,
            summary,
            format,
            yes,
            quiet,
            verbose,
        } => {
            let verbosity = if quiet {
                commands::merge::Verbosity::Quiet
            } else if verbose {
                commands::merge::Verbosity::Verbose
            } else {
                commands::merge::Verbosity::Normal
            };
            let ops = commands::batch::BatchOps {
                validate,
                validate_level: Some(validate_level),
                stats,
                list,
                convert,
                convert_ascii,
                output_dir,
            };
            let config = commands::batch::BatchConfig {
                jobs,
                recursive,
                summary,
                verbosity,
                format,
                yes,
            };
            // run() returns Ok(true) when all files succeeded, Ok(false) when any failed
            let all_succeeded = commands::batch::run(inputs, ops, config)?;
            if !all_succeeded {
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
