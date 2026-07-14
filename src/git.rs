use std::process::Command;

pub fn get_staged_diff() -> Result<String, String> {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .map_err(|e| format!("Erreur d'exécution de git diff: {}", e))?;

    if !output.status.success() {
        return Err("La commande git diff a échoué.".to_string());
    }

    let diff = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if diff.is_empty() {
        return Err("Aucun changement indexé (staged). Utilise `git add` d'abord.".to_string());
    }

    Ok(diff)
}

pub fn commit(message: &str) -> Result<(), String> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()
        .map_err(|e| format!("Erreur d'exécution de git commit: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Le commit a échoué :\n{}", err));
    }

    Ok(())
}
