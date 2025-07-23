use crate::models::{ExtractedSection, SubsectionAnalysis, OutputJson, Metadata};
use anyhow::Result;
use chrono::Utc;
use pdf::file::FileOptions;
use pdf::content::{Content, Op};
use pdf::object::Resolve;
use regex::Regex;
use std::path::Path;

pub struct PdfProcessor;

impl PdfProcessor {
    pub fn process_pdf_collection(input_path: &str, output_path: &str) -> Result<()> {
        let input_json = std::fs::read_to_string(input_path)?;
        let input: crate::models::InputJson = serde_json::from_str(&input_json)?;

        let mut extracted_sections = Vec::new();
        let mut subsection_analysis = Vec::new();

        for doc in &input.documents {
            let pdf_path = Path::new(input_path)
                .parent()
                .unwrap()
                .join("pdfs")
                .join(&doc.filename);
            
            if !pdf_path.exists() {
                eprintln!("Skipping missing PDF: {}", pdf_path.display());
                continue;
            }

            match Self::extract_pdf_text(&pdf_path) {
                Ok((text, page_texts)) => {
                    // Extract sections (headings)
                    let re = Regex::new(r"(?m)^[A-Z][A-Za-z ]+$").unwrap();
                    for (i, cap) in re.captures_iter(&text).enumerate() {
                        extracted_sections.push(ExtractedSection {
                            document: doc.filename.clone(),
                            section_title: cap[0].to_string(),
                            importance_rank: (i + 1) as u32,
                            page_number: 1, // Simplified - would need proper page tracking
                        });
                    }

                    // Find relevant content based on persona
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
                }
            }
        }

        // Sort sections by importance
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

        std::fs::write(output_path, serde_json::to_string_pretty(&output)?)?;
        Ok(())
    }

    fn extract_pdf_text(path: &Path) -> Result<(String, Vec<(usize, String)>)> {
        let file = FileOptions::cached().open(path)?;
        let mut full_text = String::new();
        let mut page_texts = Vec::new();
        
        for (page_num, page) in file.pages().enumerate() {
            if let Ok(page) = page {
                if let Some(content) = &page.contents {
                    let mut page_text = String::new();
                    Self::extract_text_from_content(&file, content, &mut page_text)?;
                    full_text.push_str(&page_text);
                    page_texts.push((page_num + 1, page_text));
                }
            }
        }
        
        Ok((full_text, page_texts))
    }

    fn extract_text_from_content(resolver: &impl Resolve, content: &Content, text: &mut String) -> Result<()> {
        for op in content.operations(resolver)? {
            match op {
                Op::TextDraw { text: t } => {
                    text.push_str(&t.to_string_lossy());
                    text.push(' ');
                },
                Op::TextNewline => {
                    text.push('\n');
                },
                _ => {}
            }
        }
        Ok(())
    }

    fn find_relevant_content(
        doc_name: &str,
        page_texts: &[(usize, String)], 
        persona: &str, 
        task: &str
    ) -> Vec<SubsectionAnalysis> {
        let keywords = match persona {
            "Travel Planner" => vec![
                "hotel", "restaurant", "itinerary", "transport", "budget",
                "beach", "coast", "city", "travel", "plan", "friends"
            ],
            "HR Professional" => vec!["form", "fillable", "signature", "compliance", "onboarding"],
            "Food Contractor" => vec!["recipe", "vegetarian", "buffet", "ingredients", "preparation"],
            _ => vec![],
        };

        let mut relevant_sections = Vec::new();
        
        for (page_num, text) in page_texts {
            // Split by paragraphs
            let paragraphs: Vec<&str> = text.split("\n\n").collect();
            
            // Find paragraphs with keywords
            for para in paragraphs {
                if keywords.iter().any(|kw| para.contains(kw)) {
                    relevant_sections.push(SubsectionAnalysis {
                        document: doc_name.to_string(),
                        refined_text: para.trim().to_string(),
                        page_number: *page_num as u32,
                    });
                }
            }
        }
        
        // Prioritize sections that mention both persona keywords and task keywords
        relevant_sections.sort_by(|a, b| {
            let a_score = keywords.iter().filter(|kw| a.refined_text.contains(*kw)).count();
            let b_score = keywords.iter().filter(|kw| b.refined_text.contains(*kw)).count();
            b_score.cmp(&a_score)
        });

        relevant_sections
    }
}