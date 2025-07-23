use serde::{Serialize, Deserialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeInfo {
    pub challenge_id: String,
    pub test_case_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub filename: String,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Persona {
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobToBeDone {
    pub task: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputJson {
    pub challenge_info: ChallengeInfo,
    pub documents: Vec<Document>,
    pub persona: Persona,
    pub job_to_be_done: JobToBeDone,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractedSection {
    pub document: String,
    pub section_title: String,
    pub importance_rank: u32,
    pub page_number: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubsectionAnalysis {
    pub document: String,
    pub refined_text: String,
    pub page_number: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub input_documents: Vec<String>,
    pub persona: String,
    pub job_to_be_done: String,
    pub processing_timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputJson {
    pub metadata: Metadata,
    pub extracted_sections: Vec<ExtractedSection>,
    pub subsection_analysis: Vec<SubsectionAnalysis>,
}