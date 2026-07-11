use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const HOOK_SCRIPT: &str = r#"#!/bin/sh
# Skills Pal — Pre-commit Hook
# Ce hook exécute `skills_pal scan` avant chaque commit.
# Si des erreurs sont trouvées, le commit est bloqué.

echo "🔍 Skills Pal: Scan pré-commit en cours..."

OUTPUT=$(skills_pal scan 2>&1)
EXIT_CODE=$?

echo "$OUTPUT"

# Vérifier si des erreurs (pas des warnings) ont été trouvées
if echo "$OUTPUT" | grep -q "\[ERROR\]"; then
    echo ""
    echo "❌ Le commit a été bloqué par Skills Pal (erreurs détectées)."
    echo "   Corrige les erreurs ci-dessus et réessaie."
    exit 1
fi

echo "✅ Skills Pal: Aucune erreur bloquante. Commit autorisé."
exit 0
"#;

fn get_git_hook_path() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| format!("Erreur lors de l'exécution de git: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "{} Aucun dépôt Git détecté dans ce dossier.\n  {} Lancez d'abord {}",
            "❌", "→".dimmed(), "git init".cyan()
        ));
    }

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let hooks_dir = Path::new(&git_dir).join("hooks");
    
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir)
            .map_err(|e| format!("Impossible de créer le dossier hooks: {}", e))?;
    }

    Ok(hooks_dir.join("pre-commit"))
}

pub fn install_hook() -> Result<(), String> {
    println!("{} Note sur la compatibilité Windows : Ce hook bash nécessite Git Bash, WSL ou MSYS2 pour fonctionner correctement sous Windows.", "ℹ".cyan());

    let hook_path = get_git_hook_path()?;

    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path).unwrap_or_default();
        if existing.contains("Skills Pal") {
            println!("{} Le hook pre-commit Skills Pal est déjà installé.", "ℹ".cyan());
            return Ok(());
        } else {
            println!("{} Un hook pre-commit existe déjà (non créé par Skills Pal).", "⚠".yellow());
            let backup_path = hook_path.with_extension("backup");
            println!("  {} Il a été remplacé. L'ancien est sauvegardé dans {}", "→".dimmed(), backup_path.display().to_string().dimmed());
            fs::copy(&hook_path, &backup_path)
                .map_err(|e| format!("Impossible de sauvegarder l'ancien hook: {}", e))?;
        }
    }

    fs::write(&hook_path, HOOK_SCRIPT)
        .map_err(|e| format!("Impossible d'écrire le hook: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&hook_path, perms)
            .map_err(|e| format!("Impossible de rendre le hook exécutable: {}", e))?;
    }

    println!("{} Hook pre-commit installé avec succès !", "✔".green());
    println!("  {} À chaque {}, Skills Pal scannera automatiquement ton code.", "→".dimmed(), "git commit".cyan().bold());
    
    Ok(())
}

pub fn uninstall_hook() -> Result<(), String> {
    let hook_path = match get_git_hook_path() {
        Ok(path) => path,
        Err(_) => {
            println!("{} Aucun dépôt Git détecté.", "ℹ".cyan());
            return Ok(());
        }
    };

    if !hook_path.exists() {
        println!("{} Aucun hook pre-commit à supprimer.", "ℹ".cyan());
        return Ok(());
    }

    let content = fs::read_to_string(&hook_path).unwrap_or_default();
    if !content.contains("Skills Pal") {
        println!("{} Le hook pre-commit existant n'a pas été créé par Skills Pal. Suppression annulée.", "⚠".yellow());
        return Ok(());
    }

    fs::remove_file(&hook_path)
        .map_err(|e| format!("Impossible de supprimer le hook: {}", e))?;

    let backup_path = hook_path.with_extension("backup");
    if backup_path.exists() {
        fs::rename(&backup_path, &hook_path)
            .map_err(|e| format!("Impossible de restaurer le backup: {}", e))?;
        println!("{} Hook Skills Pal supprimé. L'ancien hook a été restauré.", "✔".green());
    } else {
        println!("{} Hook pre-commit Skills Pal supprimé.", "✔".green());
    }

    Ok(())
}
