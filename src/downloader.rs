use reqwest::Client;
use std::fs;
use std::io::Cursor;
use zip::ZipArchive;
use crate::config::PLUGINS_DIR;

pub async fn download_skill(github_url: &str) -> Result<String, String> {
    // Transform repository URL to zip download URL
    let mut url = github_url.to_string();
    if !url.ends_with(".zip") {
        url = format!("{}/archive/refs/heads/main.zip", url.trim_end_matches('/'));
    }

    let client = Client::new();
    let response = client.get(&url)
        .header("User-Agent", "Skills-Pal-App")
        .send().await.map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to download plugin: {}", response.status()));
    }

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    let reader = Cursor::new(bytes);
    
    let mut archive = ZipArchive::new(reader).map_err(|e| e.to_string())?;
    fs::create_dir_all(PLUGINS_DIR).unwrap_or_default();
    
    // Extract zip contents
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };
        
        let outpath = std::path::Path::new(PLUGINS_DIR).join(outpath);

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).unwrap_or_default();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap_or_default();
                }
            }
            let mut outfile = fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }

    Ok("Plugin installé avec succès.".into())
}
