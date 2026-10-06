//! UORC Native CLI
//!
//! Provides the normative command surface and argument semantics as per Section 16.
//! Implements strict parsing, typed arguments, and exact exit categories.

use std::env;
use std::process;
use std::io::{self, Read, Write};
use uorc_core::api::{compress, decompress, ApiError};

// Section 16 exit semantics mapping
const EXIT_SUCCESS: i32 = 0;
const EXIT_USAGE_ERROR: i32 = 64; // Option conflicts, missing args
const EXIT_DATA_ERROR: i32 = 65; // Invalid archive format
const EXIT_OPERATIONAL_FAILURE: i32 = 70; // Synthesis/evaluation failed

fn print_usage() {
    eprintln!("Usage: uorc <COMMAND> [OPTIONS]");
    eprintln!("Commands:");
    eprintln!("  compress   --memory-limit <BYTES>");
    eprintln!("  decompress --memory-limit <BYTES>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(EXIT_USAGE_ERROR);
    }

    let command = &args[1];
    
    // Minimal semantic argument parser 
    // Enforcing strict decimal parsing and conflicts as required by Issue 1102
    let mut memory_limit: Option<usize> = None;
    
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--memory-limit" => {
                if i + 1 < args.len() {
                    match args[i+1].parse() {
                        Ok(val) => memory_limit = Some(val),
                        Err(_) => {
                            eprintln!("Error: --memory-limit must be a typed decimal integer");
                            process::exit(EXIT_USAGE_ERROR);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --memory-limit requires a value");
                    process::exit(EXIT_USAGE_ERROR);
                }
            }
            _ => {
                eprintln!("Error: Unknown argument {}", args[i]);
                process::exit(EXIT_USAGE_ERROR);
            }
        }
    }

    let memory_limit = match memory_limit {
        Some(l) => l,
        None => {
            eprintln!("Error: --memory-limit is required");
            process::exit(EXIT_USAGE_ERROR);
        }
    };

    match command.as_str() {
        "compress" => {
            let mut input = Vec::new();
            if io::stdin().read_to_end(&mut input).is_err() {
                eprintln!("Error: Failed to read from stdin stream");
                process::exit(EXIT_OPERATIONAL_FAILURE);
            }
            
            match compress(&input, memory_limit) {
                Ok(archive) => {
                    if io::stdout().write_all(&archive).is_err() {
                        process::exit(EXIT_OPERATIONAL_FAILURE);
                    }
                    process::exit(EXIT_SUCCESS);
                }
                Err(e) => {
                    eprintln!("Compression failed: {}", e);
                    process::exit(EXIT_OPERATIONAL_FAILURE);
                }
            }
        }
        "decompress" => {
            let mut input = Vec::new();
            if io::stdin().read_to_end(&mut input).is_err() {
                eprintln!("Error: Failed to read from stdin stream");
                process::exit(EXIT_OPERATIONAL_FAILURE);
            }
            
            match decompress(&input, memory_limit) {
                Ok(data) => {
                    if io::stdout().write_all(&data).is_err() {
                        process::exit(EXIT_OPERATIONAL_FAILURE);
                    }
                    process::exit(EXIT_SUCCESS);
                }
                Err(ApiError::InvalidArchive) => {
                    eprintln!("Decompression failed: Invalid archive format");
                    // Explicit exit categories
                    process::exit(EXIT_DATA_ERROR);
                }
                Err(e) => {
                    eprintln!("Decompression failed: {}", e);
                    process::exit(EXIT_OPERATIONAL_FAILURE);
                }
            }
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_usage();
            process::exit(EXIT_USAGE_ERROR);
        }
    }
}
