<div align="center">
  <h1>🧠 Skills Pal (spal)</h1>
  <p><strong>L'assistant IA ultime pour améliorer ton code, éradiquer la dette technique et recommander des plugins d'ingénierie.</strong></p>
  
  [![Rust](https://img.shields.io/badge/Rust-1.88.0-orange.svg)](https://www.rust-lang.org)
  [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
  [![Railway](https://img.shields.io/badge/Railway-Deployed-purple.svg)](https://railway.app)
</div>

<br />

## 🌟 Présentation

**Skills Pal** est un outil innovant divisé en deux parties (Architecture Client/Serveur) :

1. **Un CLI premium ultra-rapide (Client)** : Scanne ton code source localement, identifie la dette technique ou le manque d'optimisation, et interroge une IA (Mistral, OpenAI, Anthropic, Ollama) pour te recommander des plugins, des outils ou des compétences globales d'ingénierie à adopter.
2. **Un Serveur distant (Backend)** : Hébergé sur Railway, il scrape automatiquement GitHub toutes les 12h pour découvrir les nouveaux plugins créés par la communauté et met à jour sa base de données PostgreSQL pour te fournir des recommandations toujours à la pointe.

---

## ✨ Fonctionnalités Principales

- 🤖 **Analyse IA Intelligente** : Recommandations basées sur l'analyse sémantique globale de ton projet.
- ⚡ **Multi-LLM** : Compatible par défaut avec **Mistral AI**, mais supporte aussi OpenAI, Anthropic, et Ollama (en local).
- 🎨 **Expérience CLI Premium** : Interface animée avec spinners, sorties colorées, et menus interactifs (Fuzzy Select).
- 🌍 **Registre & Dashboard Web** : Le serveur découvre tout seul les plugins sur GitHub et expose un superbe Web Dashboard !
- 🚀 **Performances Natives** : Écrit intégralement en Rust. Consommation mémoire minimale et exécution instantanée.
- 🛡️ **Sécurisé & Anti-DDoS** : Serveur protégé par un Rate-Limiter (100 req/sec). Clés API stockées uniquement en local sur ton PC.
- 📦 **Installation Universelle** : Binaires autonomes disponibles pour Windows, macOS, et Linux sans besoin d'installer Rust.

---

## 🚀 Installation 

L'installation télécharge les binaires compilés (`skills_pal` et son raccourci `spal`) pour ton système. Aucun prérequis n'est nécessaire !

**Sur Mac / Linux :**
```bash
curl -fsSL https://raw.githubusercontent.com/Julien-Bui/skills_pal/main/install.sh | bash
```

**Sur Windows (PowerShell) :**
```powershell
iwr https://raw.githubusercontent.com/Julien-Bui/skills_pal/main/install.ps1 -useb | iex
```

---

## 🛠️ Utilisation du CLI

Le CLI est conçu pour être simple et modulaire grâce à de nombreux *flags*. Toutes les commandes acceptent l'argument `-v` ou `--verbose` pour afficher les logs de débogage techniques.

> **💡 Astuce :** Pour aller plus vite, tu peux remplacer la commande `skills_pal` par son raccourci `spal` dans le terminal ! (ex: `spal scan`)

### 1. Initialisation
Crée le fichier de configuration. Par défaut, Skills Pal utilise **Mistral AI**.
```bash
# Configuration locale classique (dans le dossier courant)
skills_pal init 
# (Raccourci: spal init)

# Configuration globale (utilisable partout sur ton PC) en une seule ligne !
skills_pal init --global --provider openai_compatible --api-key "TA_CLE_API"
```

### 2. Analyse et Recommandation
L'outil lit la structure de ton code, l'envoie à l'IA avec le contexte des plugins communautaires disponibles, et te suggère des outils pertinents pour l'architecture, la CI/CD ou la qualité de ton projet.
```bash
skills_pal recom
# (Raccourci: spal recom)

# Pour voir exactement le prompt envoyé à l'IA et l'URL interrogée :
skills_pal -v recom
```

### 3. Scan de la Dette Technique
Analyse le code source pour trouver la dette technique explicite (commentaires `TODO`, `FIXME`) et les avertissements de compilation (ex: Clippy pour Rust).
```bash
# Scanner le dossier courant
skills_pal scan 
# (Raccourci: spal scan)

# Scanner un dossier spécifique
skills_pal scan --path ./src/backend
```

### 4. Menu Interactif (Browse)
Affiche un menu de sélection flou (Fuzzy Select) dans ton terminal pour naviguer facilement parmi les skills disponibles sur le serveur et ouvrir leurs pages GitHub.
```bash
skills_pal browse
# (Raccourci: spal browse)
```

### 5. Git Hooks (Automatisation)
Bloque les commits qui contiennent de la dette technique ou des erreurs critiques !
```bash
# Installer le hook (s'exécutera avant chaque git commit)
skills_pal hook install
# (Raccourci: spal hook install)

# Le retirer (désactiver le blocage)
skills_pal hook uninstall
# Alias disponible :
skills_pal hook disable
```

### 6. Diagnostic (Doctor)
Vérifie en un clin d'œil que ton outil est parfaitement configuré (Clés API, Connectivité au serveur Railway, Dépôt Git).
```bash
skills_pal doctor
# (Raccourci: spal doctor)
```

### 7. Mise à Jour Automatique
Télécharge et installe automatiquement la dernière version de Skills Pal depuis Github. *(Le CLI te préviendra automatiquement à la fin d'un scan si une mise à jour est disponible !)*
```bash
skills_pal update
# (Raccourci: spal update)
```
*(Note : Si tu as installé l'outil globalement via le script d'installation, tu auras besoin des droits administrateur pour le mettre à jour : `sudo skills_pal update`)*

### 8. Nettoyage Complet (Reset)
Supprime tous les fichiers générés localement par l'outil (base de données locale, dossier des plugins téléchargés, fichiers zip temporaires et fichier de configuration). Idéal pour repartir à zéro.
```bash
skills_pal clean
# (Raccourci: spal clean)
```

---

## 🌍 Architecture & Déploiement

Ce dépôt contient deux binaires distincts :

- **Le Client CLI** (`skills_pal`) : `src/main.rs`
- **Le Serveur API** (`server`) : `src/server/main.rs`


### Déployer son propre Serveur (Railway)

Si tu souhaites héberger ta propre instance du registre de plugins :
1. Connecte ton compte Railway à ton fork de ce dépôt GitHub.
2. Provisionne une base de données **PostgreSQL**.
3. Dans les variables d'environnement de ton service web, ajoute : `DATABASE_URL=postgresql://...` (pour la BDD) et un `GITHUB_TOKEN=ghp_...` (pour permettre au scraper de contourner les limites de requêtes de GitHub).
4. Dans **Settings > Deploy** de Railway :
   - **Custom Build Command** : `cargo build --release --bin server && cp target/release/server ./server`
   - **Custom Start Command** : `./server`

Le serveur construira automatiquement les tables SQL, lancera son cache en RAM (0 latence), commencera à scraper GitHub en arrière-plan, et **servira le Dashboard Web UI** sur l'URL de ton application !

---

## 🧩 Créer un Plugin pour Skills Pal

Tu as développé un outil, une documentation ou un script génial et tu veux que Skills Pal le recommande aux autres développeurs ?
Rien de plus simple :

1. Crée un dépôt public sur GitHub.
2. Ajoute tes skills au format Markdown dans un dossier `skills/` (avec les bonnes métadonnées Frontmatter).
3. Le serveur backend de Skills Pal scannera le dossier et l'ajoutera automatiquement à sa base PostgreSQL toutes les 12h.
