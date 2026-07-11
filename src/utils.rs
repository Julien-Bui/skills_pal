use walkdir::{WalkDir, DirEntry};

pub fn check_exclusions(file_path: &str) -> bool {
    file_path.contains("target/") || file_path.contains(".git/") || file_path.contains("node_modules/") || file_path.contains(".idea/")
}

pub fn get_project_files(project_path: &str) -> Vec<DirEntry> {
    WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            if let Some(path_str) = e.path().to_str() {
                !check_exclusions(path_str)
            } else {
                false
            }
        })
        .collect()
}
