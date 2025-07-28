# Persona-Driven Document Intelligence System

**Challenge:** Round 1B: Persona-Driven Document Intelligence
**Theme:** “Connect What Matters — For the User Who Matters”

## Overview

This project is a Rust-based system built to address the Persona-Driven Document Intelligence challenge. It functions as an intelligent document analyst that processes a collection of PDFs, extracting and prioritizing the most relevant sections based on a specific user persona and their job-to-be-done. The system is designed to be generic, handling diverse domains and personas to deliver a concise, relevant, and actionable summary.

## Approach & Methodology

To meet the challenge's scoring criteria, the system's core logic focuses on accurately determining relevance at both the section and sub-section levels.

1.  **Text Extraction with OCR Fallback**: The system first attempts to parse and extract text natively from the PDF structure. If this fails or yields no text (common with image-based PDFs), it automatically falls back to an external OCR tool (`pdftotext`) to ensure content is captured.

2.  **Keyword-Driven Relevance**: To connect the documents to the user's needs, the system extracts key terms from the `persona` and `job_to_be_done` descriptions. These keywords become the basis for relevance scoring.

3.  **Sub-Section Analysis**: The extracted text is broken down into paragraphs (sub-sections). A sub-section is considered relevant if it contains **at least one keyword from both the persona and the task**. This ensures that the extracted snippets are highly focused and address both the user's role and their goal.

4.  **Section Identification and Ranking**: Section titles are identified using a series of regular expressions designed to catch common heading formats (e.g., title case, numbered headings). Each identified section is then scored based on the number of relevant sub-sections it contains. Sections with a higher concentration of relevant content are ranked higher, providing a clear, prioritized list for the user.

## Features

- **Persona-Based Relevance**: Analyzes documents based on a user persona and their task.
- **Hybrid Text Extraction**: Combines native PDF parsing with an OCR fallback for robustness.
- **Regex-Based Heading Detection**: Identifies section titles from the text content.
- **Dual-Keyword Analysis**: Finds relevant paragraphs by matching keywords from both the persona and the task.
- **Importance Ranking**: Ranks sections based on the density of relevant keywords.
- **Structured JSON Output**: Generates a detailed JSON output with metadata, ranked sections, and analyzed subsections.

## Technology Stack

- **Language**: [Rust](https://www.rust-lang.org/)
- **Core Crates**:
  - `anyhow`: For flexible and easy-to-use error handling.
  - `serde` & `serde_json`: For serializing and deserializing the input and output JSON.
  - `pdf`: For native PDF parsing and text extraction.
  - `regex`: For pattern matching to identify headings.
  - `chrono`: For timestamping the processing metadata.
- **External Dependencies**:
  - **Poppler**: The `pdftotext` tool is required for the OCR fallback.

## Project Constraints

This project adheres to the specified constraints:
- **CPU Only**: All processing is handled on the CPU.
- **No Internet Access**: The application runs entirely offline.
- **Lightweight**: The compiled binary and its dependencies are minimal, well under the 1GB model size limit.

## Setup and Usage

### Prerequisites

- **Rust**: Install the Rust toolchain from [rustup.rs](https://rustup.rs/). Requires Rust 1.70+ (2021 edition)
- **Poppler** (Optional but recommended): For enhanced OCR fallback capability
  - **Windows**: Download and install [Poppler for Windows](https://github.com/oschwartz10612/poppler-windows)
  - **macOS**: `brew install poppler`
  - **Linux**: `sudo apt-get install poppler-utils` (Ubuntu/Debian) or equivalent

### Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd Adobe1B
   ```

2. **Verify Rust installation**:
   ```bash
   rustc --version
   cargo --version
   ```

3. **Build the project**:
   ```bash
   # For development
   cargo build
   
   # For optimized release build
   cargo build --release
   ```

### Project Structure

```
Adobe1B/
├── collections/              # Document collections
│   ├── adobe_learning/       # Adobe Acrobat tutorials
│   │   ├── pdfs/            # PDF files
│   │   ├── challenge1b_input.json
│   │   └── challenge1b_output.json (generated)
│   ├── recipe_collection/    # Cooking recipes
│   └── travel_planning/      # Travel guides
├── src/                     # Source code
├── target/                  # Compiled binaries (generated)
└── Cargo.toml              # Dependencies
```

### Running the Application

#### Method 1: Using Cargo (Development)
```bash
# Run from project root directory
cargo run
```

#### Method 2: Using Compiled Binary (Production)
```bash
# After building with --release
./target/release/pdf_analyzer        # Unix/Linux/macOS
./target/release/pdf_analyzer.exe    # Windows
```

#### Method 3: Check and Test
```bash
# Verify compilation without running
cargo check

# Run tests (if available)
cargo test

# Run with verbose output
cargo run --verbose
```

### Input File Format

Each collection requires a `challenge1b_input.json` file:

```json
{
  "challenge_info": {
    "challenge_id": "challenge1b",
    "test_case_name": "pdf_content_extraction",
    "description": "Extract relevant content based on persona and task"
  },
  "documents": [
    {
      "filename": "example.pdf",
      "title": "Example Document"
    }
  ],
  "persona": {
    "role": "Travel Planner"
  },
  "job_to_be_done": {
    "task": "Plan a trip to South of France with friends"
  }
}
```

### Supported Personas and Use Cases

| Persona | Keywords | Example Task |
|---------|----------|--------------|
| **Travel Planner** | hotel, restaurant, itinerary, transport, budget, accommodation | Plan a trip to South of France |
| **HR Professional** | form, fillable, signature, compliance, onboarding, employee | Process employee onboarding documents |
| **Food Contractor** | recipe, vegetarian, buffet, ingredients, preparation, menu | Plan a buffet menu for an event |

### Expected Output

The application generates `challenge1b_output.json` with:
- **Metadata**: Processing timestamp, input documents, persona, and task
- **Extracted Sections**: Identified document sections with importance ranking
- **Subsection Analysis**: Relevant text snippets with page numbers

### Troubleshooting

#### Common Issues

1. **Compilation Errors**:
   ```bash
   cargo clean && cargo build
   ```

2. **PDF Processing Failures**:
   - Check PDF file accessibility
   - Verify Poppler installation for OCR fallback
   - Review debug output in generated `.txt` files

3. **No Output Generated**:
   - Ensure `challenge1b_input.json` exists in collection directories
   - Verify PDF files are in `pdfs/` subdirectories
   - Check file permissions

4. **Performance Issues**:
   - Large PDFs may take time to process
   - Use `--release` build for better performance
   - Monitor memory usage for very large documents

#### Debug Information

The application provides extensive debug output:
- Console logs showing processing progress
- Generated `.txt` files alongside PDFs for manual inspection
- Detailed error messages with context

#### System Requirements

- **Memory**: Varies by PDF size (typically 100MB-1GB)
- **Disk Space**: Additional space for debug text files
- **CPU**: Any modern processor (CPU-only processing)

### Example Commands

```bash
# Basic run
cargo run

# Build and run optimized version
cargo build --release && ./target/release/pdf_analyzer

# Check for errors without running
cargo check

# Clean and rebuild
cargo clean && cargo build

# Run with environment variables for debugging
RUST_LOG=debug cargo run
```

### Integration Notes

- The application processes all collections in the `collections/` directory automatically
- Each collection is processed independently
- Output files are generated in the same directory as input files
- The system is designed to be generic and handle various document types and personas
