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

        let persona_keywords = Self::extract_keywords_from_text(&input.persona.role);
        let task_keywords = Self::extract_keywords_from_text(&input.job_to_be_done.task);

        for doc in &input.documents {
            let pdf_path = Path::new(input_path).parent().unwrap().join("pdfs").join(&doc.filename);
            if !pdf_path.exists() {
                return Err(anyhow::anyhow!("PDF not found at: {}", pdf_path.display()));
            }

            match Self::extract_pdf_text(&pdf_path) {
                Ok((_full_text, page_texts)) => {
                    for (page_num, page_text) in &page_texts {
                        let headings = Self::extract_headings_from_page(page_text);
                        for heading in headings {
                            extracted_sections.push(ExtractedSection {
                                document: doc.filename.clone(),
                                section_title: heading,
                                importance_rank: 0, // Placeholder, will be updated later
                                page_number: *page_num as u32,
                            });
                        }
                    }

                    let relevant_content = Self::find_relevant_content(
                        &doc.filename,
                        &page_texts,
                        &persona_keywords,
                        &task_keywords,
                    );
                    subsection_analysis.extend(relevant_content);
                }
                Err(e) => {
                    eprintln!("Error processing {}: {}", pdf_path.display(), e);
                    // Try OCR as fallback
                    match Self::extract_with_ocr(&pdf_path) {
                        Ok(ocr_text) => {
                            println!("[INFO] Using OCR-extracted text for {}", pdf_path.display());
                            let page_texts = vec![(1, ocr_text.clone())]; // Treat OCR output as a single page
                            subsection_analysis.extend(Self::find_relevant_content(
                                &doc.filename,
                                &page_texts,
                                &persona_keywords,
                                &task_keywords
                            ));
                        }
                        Err(ocr_err) => {
                            eprintln!("OCR also failed for {}: {}", pdf_path.display(), ocr_err);
                        }
                    }
                }
            }
        }

        Self::rank_sections(&mut extracted_sections, &subsection_analysis, &persona_keywords, &task_keywords);

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
        let file = FileOptions::cached().open(path)?;
        let mut full_text = String::new();
        let mut page_texts = Vec::new();
        
        for page_num in 0..file.num_pages() {
            let page = file.get_page(page_num)?;
            let mut page_text = String::new();
            
            if let Some(content) = &page.contents {
                Self::extract_text_from_content(&file, content, &mut page_text)?;
            }
            
            let cleaned_text = Self::clean_extracted_text(&page_text);
            if !cleaned_text.is_empty() {
                full_text.push_str(&cleaned_text);
                full_text.push_str("\n\n");
                page_texts.push((page_num as usize + 1, cleaned_text));
            }
        }
        
        if full_text.trim().is_empty() {
            return Err(anyhow::anyhow!("No text extracted from PDF - will try OCR"));
        }
        
        Ok((full_text, page_texts))
    }

    fn clean_extracted_text(raw_text: &str) -> String {
        let cleaned = raw_text.lines().map(|line| line.trim()).filter(|line| !line.is_empty()).collect::<Vec<_>>().join(" ");
        let re = Regex::new(r"\s+").unwrap();
        re.replace_all(&cleaned, " ").to_string()
    }

    fn extract_text_from_content(resolver: &impl Resolve, content: &Content, text: &mut String) -> Result<()> {
        for op in content.operations(resolver)? {
            match op {
                Op::TextDraw { text: t } => {
                    text.push_str(&t.to_string_lossy());
                }
                Op::TextDrawAdjusted { array } => {
                    for item in array {
                        if let pdf::content::TextDrawAdjusted::Text(text_str) = item {
                            text.push_str(&text_str.to_string_lossy());
                        }
                    }
                }
                Op::TextNewline => {
                    text.push('\n');
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn extract_with_ocr(path: &Path) -> Result<String> {
        let output = Command::new("pdftotext")
            .arg("-layout")
            .arg(path)
            .arg("-")
            .output()
            .with_context(|| "Failed to execute pdftotext. Is poppler-utils installed?")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("OCR failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        String::from_utf8(output.stdout).with_context(|| "OCR output not valid UTF-8")
    }

    fn extract_keywords_from_text(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty() && s.len() > 2)
            .collect()
    }

    fn extract_headings_from_page(page_text: &str) -> Vec<String> {
        let heading_patterns = [
            r"(?m)^([A-Z][A-Za-z\s]{3,}):?$",
            r"(?m)^(\d+\.?\s+[A-Z][A-Za-z\s]+):?$",
            r"(?m)^(Chapter\s+\d+[^.]*):?$",
            r"(?m)^([A-Z\s]{4,}):?$",
        ];
        let mut headings = Vec::new();
        for pattern in &heading_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for cap in re.captures_iter(page_text) {
                    if let Some(heading_match) = cap.get(1) {
                        headings.push(heading_match.as_str().trim().to_string());
                    }
                }
            }
        }
        headings
    }

    fn rank_sections(sections: &mut [ExtractedSection], analysis: &[SubsectionAnalysis], persona_keywords: &[String], task_keywords: &[String]) {
        for section in sections.iter_mut() {
            let mut score = 0;
            for analyzed_part in analysis {
                if analyzed_part.document == section.document && analyzed_part.page_number == section.page_number {
                    let text_lower = analyzed_part.refined_text.to_lowercase();
                    score += persona_keywords.iter().filter(|k| text_lower.contains(*k)).count();
                    score += task_keywords.iter().filter(|k| text_lower.contains(*k)).count();
                }
            }
            section.importance_rank = score as u32;
        }
        sections.sort_by(|a, b| b.importance_rank.cmp(&a.importance_rank));
        for (i, section) in sections.iter_mut().enumerate() {
            section.importance_rank = (i + 1) as u32;
        }
    }

    fn find_relevant_content(
        doc_name: &str,
        page_texts: &[(usize, String)],
        persona_keywords: &[String],
        task_keywords: &[String],
    ) -> Vec<SubsectionAnalysis> {
        let mut relevant_sections = Vec::new();
        for (page_num, text) in page_texts {
            let paragraphs: Vec<String> = text.split("\n\n").map(|s| s.to_string()).collect();
            for para in paragraphs {
                let para_lower = para.to_lowercase();
                let persona_matches = persona_keywords.iter().any(|k| para_lower.contains(k));
                let task_matches = task_keywords.iter().any(|k| para_lower.contains(k));

                if persona_matches && task_matches {
                    println!("[DEBUG] Found relevant paragraph on page {} of {}: '{}'", page_num, doc_name, para.chars().take(100).collect::<String>());
                    relevant_sections.push(SubsectionAnalysis {
                        document: doc_name.to_string(),
                        refined_text: para.trim().to_string(),
                        page_number: *page_num as u32,
                    });
                }
            }
        }

        relevant_sections
    }
}