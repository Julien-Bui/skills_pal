use colored::Colorize;
use std::fs;
use std::path::Path;

const HOOK_PATH: &str = ".git/hooks/pre-commit";

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

pub fn install_hook() -> Result<(), String> {
    // Vérifier qu'on est dans un dépôt Git
    if !Path::new(".git").exists() {
        return Err(format!(
            "{} Aucun dépôt Git détecté dans ce dossier.\n  {} Lancez d'abord {}",
            "❌", "→".dimmed(), "git init".cyan()
        ));
    }

    // Créer le dossier hooks s'il n'existe pas
    let hooks_dir = Path::new(".git/hooks");
    if !hooks_dir.exists() {
        fs::create_dir_all(hooks_dir)
            .map_err(|e| format!("Impossible de créer .git/hooks: {}", e))?;
    }

    // Vérifier si un hook existe déjà
    if Path::new(HOOK_PATH).exists() {
        let existing = fs::read_to_string(HOOK_PATH).unwrap_or_default();
        if existing.contains("Skills Pal") {
            println!("{} Le hook pre-commit Skills Pal est déjà installé.", "ℹ".cyan());
            return Ok(());
        } else {
            println!("{} Un hook pre-commit existe déjà (non créé par Skills Pal).", "⚠".yellow());
            println!("  {} Il a été remplacé. L'ancien est sauvegardé dans {}", "→".dimmed(), ".git/hooks/pre-commit.backup".dimmed());
            fs::copy(HOOK_PATH, ".git/hooks/pre-commit.backup")
                .map_err(|e| format!("Impossible de sauvegarder l'ancien hook: {}", e))?;
        }
    }

    // Écrire le hook
    fs::write(HOOK_PATH, HOOK_SCRIPT)
        .map_err(|e| format!("Impossible d'écrire le hook: {}", e))?;

    // Rendre le hook exécutable (Unix uniquement)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(HOOK_PATH, perms)
            .map_err(|e| format!("Impossible de rendre le hook exécutable: {}", e))?;
    }

    println!("{} Hook pre-commit installé avec succès !", "✔".green());
    println!("  {} À chaque {}, Skills Pal scannera automatiquement ton code.", "→".dimmed(), "git commit".cyan().bold());
    
    Ok(())
}

pub fn uninstall_hook() -> Result<(), String> {
    if !Path::new(HOOK_PATH).exists() {
        println!("{} Aucun hook pre-commit à supprimer.", "ℹ".cyan());
        return Ok(());
    }

    // Vérifier que c'est bien un hook Skills Pal
    let content = fs::read_to_string(HOOK_PATH).unwrap_or_default();
    if !content.contains("Skills Pal") {
        println!("{} Le hook pre-commit existant n'a pas été créé par Skills Pal. Suppression annulée.", "⚠".yellow());
        return Ok(());
    }

    fs::remove_file(HOOK_PATH)
        .map_err(|e| format!("Impossible de supprimer le hook: {}", e))?;

    // Restaurer le backup s'il existe
    if Path::new(".git/hooks/pre-commit.backup").exists() {
        fs::rename(".git/hooks/pre-commit.backup", HOOK_PATH)
            .map_err(|e| format!("Impossible de restaurer le backup: {}", e))?;
        println!("{} Hook Skills Pal supprimé. L'ancien hook a été restauré.", "✔".green());
    } else {
        println!("{} Hook pre-commit Skills Pal supprimé.", "✔".green());
    }

    Ok(())
}
