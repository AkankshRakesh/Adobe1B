use crate::models::{ExtractedSection, SubsectionAnalysis, OutputJson, Metadata};
use anyhow::{Context, Result};
use chrono::Utc;
use pdf::file::FileOptions;
use pdf::content::{Content, Op};
use pdf::object::Resolve;
use regex::Regex;
use std::path::Path;
use std::process::Command;

pub struct PdfProcessor;

impl PdfProcessor {
    pub fn process_pdf_collection(input_path: &str, output_path: &str) -> Result<()> {
        let input_json = std::fs::read_to_string(input_path)
            .with_context(|| format!("Failed to read input JSON at {}", input_path))?;
        let input: crate::models::InputJson = serde_json::from_str(&input_json)
            .with_context(|| format!("Failed to parse input JSON at {}", input_path))?;

        let mut extracted_sections = Vec::new();
        let mut subsection_analysis = Vec::new();

        for doc in &input.documents {
            let pdf_path = Path::new(input_path)
                .parent()
                .unwrap()
                .join("pdfs")
                .join(&doc.filename);
            
            if !pdf_path.exists() {
                return Err(anyhow::anyhow!("PDF not found at: {}", pdf_path.display()));
            }

            match Self::extract_pdf_text(&pdf_path) {
                Ok((text, page_texts)) => {
                    // Enhanced heading detection with multiple patterns
                    let heading_patterns = vec![
                        r"(?m)^([A-Z][A-Za-z\s]{3,}):?\s*$",           // Capitalized headings
                        r"(?m)^(\d+\.?\s+[A-Z][A-Za-z\s]+):?\s*$",     // Numbered headings
                        r"(?m)^(Chapter\s+\d+[^.]*):?\s*$",            // Chapter headings
                        r"(?m)^([A-Z\s]{4,}):?\s*$",                   // ALL CAPS headings
                        r"(?m)^([A-Z][a-z]+\s+[A-Z][a-z]+.*):?\s*$",   // Title Case headings
                    ];

                    for pattern in heading_patterns {
                        if let Ok(re) = Regex::new(pattern) {
                            for (_i, cap) in re.captures_iter(&text).enumerate() {
                                if let Some(heading_match) = cap.get(1) {
                                    let heading = heading_match.as_str().trim();
                                    if heading.len() > 3 && heading.len() < 100 {
                                        println!("[DEBUG] Found heading: '{}'", heading);
                                        extracted_sections.push(ExtractedSection {
                                            document: doc.filename.clone(),
                                            section_title: heading.to_string(),
                                            importance_rank: (extracted_sections.len() + 1) as u32,
                                            page_number: 1, // Will be improved with page tracking
                                        });
                                    }
                                }
                            }
                        }
                    }

                    let relevant_content = Self::find_relevant_content(
                        &doc.filename,
                        &page_texts,
                        &input.persona.role,
                        &input.job_to_be_done.task
                    );
                    subsection_analysis.extend(relevant_content);
                }
                Err(e) => {
                    eprintln!("Error processing {}: {}", pdf_path.display(), e);
                    // Try OCR as fallback
                    match Self::extract_with_ocr(&pdf_path) {
                        Ok(ocr_text) => {
                            println!("[INFO] Using OCR-extracted text for {}", pdf_path.display());
                            let page_texts = vec![(1, ocr_text.clone())];
                            subsection_analysis.extend(Self::find_relevant_content(
                                &doc.filename,
                                &page_texts,
                                &input.persona.role,
                                &input.job_to_be_done.task
                            ));
                        }
                        Err(ocr_err) => {
                            eprintln!("OCR also failed for {}: {}", pdf_path.display(), ocr_err);
                        }
                    }
                }
            }
        }

        extracted_sections.sort_by_key(|s| s.importance_rank);

        let output = OutputJson {
            metadata: Metadata {
                input_documents: input.documents.iter().map(|d| d.filename.clone()).collect(),
                persona: input.persona.role.clone(),
                job_to_be_done: input.job_to_be_done.task.clone(),
                processing_timestamp: Utc::now().to_rfc3339(),
            },
            extracted_sections,
            subsection_analysis,
        };

        std::fs::write(output_path, serde_json::to_string_pretty(&output)?)
            .with_context(|| format!("Failed to write output to {}", output_path))?;
        
        Ok(())
    }

    fn extract_pdf_text(path: &Path) -> Result<(String, Vec<(usize, String)>)> {
        println!("[DEBUG] Opening PDF: {}", path.display());
        let file = FileOptions::cached().open(path)?;
        let mut full_text = String::new();
        let mut page_texts = Vec::new();
        
        let total_pages = file.num_pages();
        println!("[DEBUG] PDF has {} pages", total_pages);
        
        for page_num in 0..total_pages {
            let page_num_usize = page_num as usize;
            match file.get_page(page_num) {
                Ok(page) => {
                    let mut page_text = String::new();
                    
                    // Try to extract from page contents
                    if let Some(content) = &page.contents {
                        if let Err(e) = Self::extract_text_from_content(&file, content, &mut page_text) {
                            println!("[WARN] Failed to extract from page {} content: {}", page_num + 1, e);
                        }
                    }
                    
                    // Also try to extract from page resources/annotations if content is empty
                    if page_text.trim().is_empty() {
                        // Try alternative text extraction methods
                        if let Some(resources) = &page.resources {
                            // Try to extract text from XObjects or other resources
                            for (_, _resource) in &resources.xobjects {
                                // Handle XObject text extraction if needed
                                println!("[DEBUG] Found XObject on page {}", page_num_usize + 1);
                                // Note: XObject text extraction would require more complex handling
                            }
                        }
                    }
                    
                    // Clean and process the extracted text
                    let cleaned_text = Self::clean_extracted_text(&page_text);
                    
                    if !cleaned_text.is_empty() {
                        println!("[DEBUG] Page {} extracted {} chars", page_num_usize + 1, cleaned_text.len());
                        // Show first 100 chars for debugging - safe substring
                        let preview = if cleaned_text.len() > 100 {
                            let mut end = 100;
                            while end > 0 && !cleaned_text.is_char_boundary(end) {
                                end -= 1;
                            }
                            format!("{}...", &cleaned_text[..end])
                        } else {
                            cleaned_text.clone()
                        };
                        println!("[DEBUG] Page {} preview: {}", page_num_usize + 1, preview);
                        
                        full_text.push_str(&cleaned_text);
                        full_text.push_str("\n\n"); // Maintain page separation
                        page_texts.push((page_num_usize + 1, cleaned_text));
                    } else {
                        println!("[WARN] No text extracted from page {}", page_num_usize + 1);
                    }
                }
                Err(e) => {
                    println!("[ERROR] Failed to get page {}: {}", page_num_usize + 1, e);
                }
            }
        }
        
        if full_text.trim().is_empty() {
            println!("[INFO] No text extracted via PDF parsing, trying OCR fallback");
            return Err(anyhow::anyhow!("No text extracted from PDF - will try OCR"));
        }
        
        // Write debug file
        let debug_path = path.with_extension("txt");
        if let Err(e) = std::fs::write(&debug_path, &full_text) {
            println!("[WARN] Could not write debug file: {}", e);
        } else {
            println!("[DEBUG] Raw text saved to: {}", debug_path.display());
        }
        
        Ok((full_text, page_texts))
    }

    fn clean_extracted_text(raw_text: &str) -> String {
        // Remove excessive whitespace and normalize
        let cleaned = raw_text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        
        // Replace multiple spaces with single space
        let re = Regex::new(r"\s+").unwrap();
        let normalized = re.replace_all(&cleaned, " ");
        
        // Split into sentences and rejoin to maintain readability
        normalized
            .split('.')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(". ")
            + if normalized.ends_with('.') { "" } else { "." }
    }

    fn extract_text_from_content(resolver: &impl Resolve, content: &Content, text: &mut String) -> Result<()> {
        for op in content.operations(resolver)? {
            match op {
                Op::TextDraw { text: t } => {
                    let text_str = t.to_string_lossy();
                    if !text_str.trim().is_empty() {
                        text.push_str(&text_str);
                        text.push(' ');
                    }
                },
                Op::TextDrawAdjusted { array } => {
                    // Handle adjusted text drawing - array contains mixed text and adjustments
                    for item in array {
                        match item {
                            pdf::content::TextDrawAdjusted::Text(text_str) => {
                                let text_content = text_str.to_string_lossy();
                                if !text_content.trim().is_empty() {
                                    text.push_str(&text_content);
                                    text.push(' ');
                                }
                            },
                            pdf::content::TextDrawAdjusted::Spacing(_) => {
                                // Handle spacing adjustments - just add a space
                                text.push(' ');
                            }
                        }
                    }
                },
                Op::TextNewline => {
                    text.push('\n');
                },
                Op::MoveTextPosition { translation } => {
                    // Large vertical movements typically indicate paragraph breaks
                    if translation.y.abs() > 12.0 {
                        text.push('\n');
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn extract_with_ocr(path: &Path) -> Result<String> {
        println!("[INFO] Attempting OCR for: {}", path.display());
        
        let output = Command::new("pdftotext")
            .arg("-layout")  // Maintain layout
            .arg("-enc")     // Force UTF-8
            .arg("UTF-8")
            .arg(path)
            .arg("-")        // Output to stdout
            .output()
            .with_context(|| "Failed to execute pdftotext. Is poppler-utils installed?")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "OCR failed with status: {}\nError: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        let text = String::from_utf8(output.stdout)
            .with_context(|| "OCR output not valid UTF-8")?;
        
        if text.trim().is_empty() {
            return Err(anyhow::anyhow!("OCR extracted no text"));
        }
        
        Ok(text)
    }

    fn find_relevant_content(
        doc_name: &str,
        page_texts: &[(usize, String)],
        persona: &str,
        task: &str
    ) -> Vec<SubsectionAnalysis> {
        let keywords = match persona.to_lowercase().as_str() {
            "travel planner" => vec![
                "hotel", "restaurant", "itinerary", "transport", "budget",
                "beach", "coast", "city", "travel", "plan", "friends",
                "day trip", "accommodation", "sightseeing", "tour",
                "itinerary", "flight", "train", "booking", "reservation"
            ],
            "hr professional" => vec![
                "form", "fillable", "signature", "compliance", "onboarding",
                "field", "text box", "checkbox", "dropdown", "required",
                "document", "approval", "electronic", "sign", "pdf",
                "employee", "new hire", "paperwork", "tax form", "contract"
            ],
            "food contractor" => vec![
                "recipe", "vegetarian", "buffet", "ingredients", "preparation",
                "gluten-free", "menu", "dish", "cooking", "serving",
                "allergy", "dietary", "vegan", "meal", "course",
                "appetizer", "main course", "dessert", "salad", "soup"
            ],
            _ => vec![]
        };
        
        let task_keywords: Vec<String> = task.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mut relevant_sections = Vec::new();
        
        println!("[DEBUG] Searching for keywords in: {}", doc_name);
        for (page_num, text) in page_texts {
            // Split into meaningful chunks - try different splitting strategies
            let mut paragraphs: Vec<String> = Vec::new();
            
            // First, try splitting by double newlines
            let double_newline_paras: Vec<&str> = text.split("\n\n")
                .filter(|p| p.len() > 20)
                .collect();
            
            if double_newline_paras.len() > 2 {
                paragraphs.extend(double_newline_paras.iter().map(|s| s.to_string()));
            } else {
                // Fallback: split by sentence groups (3+ sentences)
                let sentences: Vec<&str> = text.split('.')
                    .filter(|s| s.trim().len() > 15)
                    .collect();
                
                for chunk in sentences.chunks(3) {
                    let para = chunk.join(". ") + ".";
                    if para.len() > 50 {
                        paragraphs.push(para);
                    }
                }
            }
            
            // If still no good paragraphs, use the whole text as one paragraph
            if paragraphs.is_empty() && text.len() > 50 {
                paragraphs.push(text.clone());
            }
            
            println!("[DEBUG] Page {} has {} meaningful paragraphs/sections", page_num, paragraphs.len());
            
            for (para_idx, para) in paragraphs.iter().enumerate() {
                let para_lower = para.to_lowercase();
                let keyword_matches: Vec<&str> = keywords.iter()
                    .filter(|kw| para_lower.contains(*kw))
                    .copied()
                    .collect();
                
                let task_matches: Vec<&str> = task_keywords.iter()
                    .filter(|kw| para_lower.contains(kw.as_str()))
                    .map(|s| s.as_str())
                    .collect();
                
                if !keyword_matches.is_empty() || !task_matches.is_empty() {
                    let relevance_score = keyword_matches.len() + task_matches.len();
                    println!("[MATCH] Page {}, Section {}: Found relevant content (score: {}) with keywords: {:?} and task terms: {:?}",
                        page_num, para_idx + 1, relevance_score, keyword_matches, task_matches);
                    
                    // Show a preview of the matched content - safe substring
                    let preview = if para.len() > 200 {
                        let mut end = 200;
                        while end > 0 && !para.is_char_boundary(end) {
                            end -= 1;
                        }
                        format!("{}...", &para[..end])
                    } else {
                        para.clone()
                    };
                    println!("[PREVIEW] {}", preview);
                    
                    relevant_sections.push(SubsectionAnalysis {
                        document: doc_name.to_string(),
                        refined_text: para.trim().to_string(),
                        page_number: *page_num as u32,
                    });
                }
            }
        }
        
        // Prioritize sections with both persona and task keywords
        relevant_sections.sort_by(|a, b| {
            let a_score = keywords.iter().filter(|kw| a.refined_text.to_lowercase().contains(*kw)).count()
                + task_keywords.iter().filter(|kw| a.refined_text.to_lowercase().contains(kw.as_str())).count();
            
            let b_score = keywords.iter().filter(|kw| b.refined_text.to_lowercase().contains(*kw)).count()
                + task_keywords.iter().filter(|kw| b.refined_text.to_lowercase().contains(kw.as_str())).count();
            
            b_score.cmp(&a_score)
        });

        relevant_sections
    }
}