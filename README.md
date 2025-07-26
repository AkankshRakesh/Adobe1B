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

- **Rust**: Install the Rust toolchain from [rustup.rs](https://rustup.rs/).
- **Poppler**: For OCR fallback, install Poppler from your package manager (e.g., `brew install poppler`, `sudo apt-get install poppler-utils`).

### Running the Application

1.  **Structure Your Data**: Place your document collections in the `collections` directory. Each collection should be a subdirectory containing a `pdfs` folder and a `challenge1b_input.json` file.
2.  **Execute from the Root**: Run the application from the project's root directory.

    ```bash
    cargo run
    ```

The system will automatically process all collections and generate a `challenge1b_output.json` file for each one.
