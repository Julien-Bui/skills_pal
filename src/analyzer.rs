use crate::models::Report;
use std::process::Command;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::utils::get_project_files;

pub async fn scan_project(project_path: String) -> Result<Vec<Report>, String> {
    let mut reports = Vec::new();
    
    // 1. VRAIE ANALYSE RUST (Via cargo clippy)
    run_clippy_analysis(&project_path, &mut reports);

    // 2. VRAIE ANALYSE DETTE TECHNIQUE (Recherche de TODO/FIXME)
    run_todo_fixme_scan(&project_path, &mut reports);

    Ok(reports)
}

fn run_clippy_analysis(project_path: &str, reports: &mut Vec<Report>) {
    let output = Command::new("cargo")
        .arg("clippy")
        .arg("--message-format=json")
        .current_dir(project_path)
        .output();
        
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
                if parsed["reason"].as_str() == Some("compiler-message") {
                    let msg = &parsed["message"];
                    let severity = msg["level"].as_str().unwrap_or("info");
                    
                    if severity == "warning" || severity == "error" {
                        let text = msg["message"].as_str().unwrap_or("Unknown issue").to_string();
                        let code = msg["code"]["code"].as_str().unwrap_or("").to_string();
                        
                        let spans = msg["spans"].as_array();
                        let mut file_path = "Unknown".to_string();
                        let mut line_number = None;
                        
                        if let Some(spans) = spans {
                            if let Some(primary_span) = spans.iter().find(|s| s["is_primary"].as_bool() == Some(true)) {
                                file_path = primary_span["file_name"].as_str().unwrap_or("Unknown").to_string();
                                line_number = primary_span["line_start"].as_i64().map(|v| v as i32);
                            }
                        }
                        
                        reports.push(Report {
                            id: None,
                            skill_id: 1, 
                            file_path,
                            line_number,
                            message: format!("Clippy: {}", text),
                            severity: severity.to_string(),
                            details: serde_json::json!({"code": code}),
                        });
                    }
                }
            }
        }
    }
}

fn run_todo_fixme_scan(project_path: &str, reports: &mut Vec<Report>) {
    for entry in get_project_files(project_path) {
        let path_str = entry.path().to_str().unwrap_or("");
        
        if entry.file_type().is_file() && (path_str.ends_with(".rs") || path_str.ends_with(".js") || path_str.ends_with(".ts") || path_str.ends_with(".jsx") || path_str.ends_with(".tsx") || path_str.ends_with(".vue") || path_str.ends_with(".svelte") || path_str.ends_with(".html") || path_str.ends_with(".css")) {
            if let Ok(file) = File::open(entry.path()) {
                let reader = BufReader::new(file);
                for (i, line_res) in reader.lines().enumerate() {
                    if let Ok(line) = line_res {
                        if line.contains("TODO") || line.contains("FIXME") {
                            let rel_path = entry.path().strip_prefix(project_path).unwrap_or(entry.path());
                            reports.push(Report {
                                id: None,
                                skill_id: 2, 
                                file_path: rel_path.display().to_string().replace("\\", "/"),
                                line_number: Some((i + 1) as i32),
                                message: "Dette technique: Commentaire TODO ou FIXME trouvé.".to_string(),
                                severity: "warning".to_string(),
                                details: serde_json::json!({"snippet": line.trim()}),
                            });
                        }
                    }
                }
            }
        }
    }
}
