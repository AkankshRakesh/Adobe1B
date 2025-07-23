mod models;
mod pdf_processor;

use anyhow::Result;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input_json_path> <output_json_path>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    pdf_processor::PdfProcessor::process_pdf_collection(input_path, output_path)
}