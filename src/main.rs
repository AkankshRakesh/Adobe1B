mod config;
mod models;
mod pdf_processor;

use anyhow::Result;

fn main() -> Result<()> {
    let config = config::Config::new()?;
    let collections = config.get_collection_paths()?;

    for (name, input_path, output_path) in collections {
        println!("Processing collection: {}", name);
        pdf_processor::PdfProcessor::process_pdf_collection(
            &input_path.to_string_lossy(),
            &output_path.to_string_lossy()
        )?;
    }

    Ok(())
}