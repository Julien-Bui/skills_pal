<div align="center">
  <h1>🧠 Skills Pal</h1>
  <p><strong>L'assistant IA ultime pour éradiquer la dette technique et recommander des plugins de productivité.</strong></p>
  
  [![Rust](https://img.shields.io/badge/Rust-1.88.0-orange.svg)](https://www.rust-lang.org)
  [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
  [![Railway](https://img.shields.io/badge/Railway-Deployed-purple.svg)](https://railway.app)
</div>

<br />

## 🌟 Présentation

**Skills Pal** est un outil innovant divisé en deux parties (Architecture Client/Serveur) :

1. **Un CLI ultra-rapide (Client)** : Scanne ton code source localement, identifie la dette technique ou le manque d'optimisation, et interroge une IA (OpenAI / Mistral) pour te recommander des plugins ou des compétences à adopter.
2. **Un Serveur distant (Backend)** : Hébergé sur Railway, il scrape automatiquement GitHub toutes les 12h pour découvrir les nouveaux plugins créés par la communauté et met à jour sa base de données PostgreSQL pour te fournir des recommandations toujours à la pointe.

Fini le code monolithique et obsolète. Laisse l'IA te guider vers les meilleurs outils de l'écosystème open-source !

---

## ✨ Fonctionnalités Principales

- 🤖 **Analyse IA Intelligente** : Fournit des recommandations de plugins basées sur l'analyse sémantique de ton code via LLM.
- ⚡ **Multi-LLM** : Compatible avec les API OpenAI et Mistral AI.
- 🌍 **Registre Communautaire Auto-Géré** : Le serveur découvre tout seul les plugins sur GitHub via le tag `skills-pal-plugin`.
- 🚀 **Performances Natives** : Écrit intégralement en Rust. Consommation mémoire minimale et exécution instantanée.
- 🛡️ **Sécurisé & Anti-DDoS** : Serveur protégé par un Rate-Limiter (100 req/sec) et sans injections SQL possibles.
- 📦 **Installation Universelle** : Binaires autonomes disponibles pour Windows, macOS, et Linux sans besoin d'installer Rust.

---

## 🚀 Installation Universelle (La plus simple)

Notre script d'installation magique télécharge le binaire compilé pour ton système. Aucun prérequis (ni Rust, ni Node, ni Python) n'est nécessaire !

**Sur Mac / Linux :**
```bash
curl -fsSL https://raw.githubusercontent.com/Julien-Bui/skills_pal/main/install.sh | bash
```

**Sur Windows (PowerShell) :**
```powershell
iwr https://raw.githubusercontent.com/Julien-Bui/skills_pal/main/install.ps1 -useb | iex
```

Une fois installé, tu peux lancer la configuration initiale n'importe où :
```bash
skills_pal init
```

---

## 🛠️ Utilisation du CLI

Le CLI est conçu pour être simple et direct. Voici les commandes principales :

### 1. Initialisation
Crée le fichier de configuration `.skillspal.toml` à la racine de ton projet.
```bash
skills_pal init
```
*N'oublie pas d'y renseigner ta clé API (OpenAI ou Mistral) et l'URL de ton serveur Railway (`registry_url`).*

### 2. Analyse et Recommandation
L'outil va lire ton code, l'envoyer à l'IA avec le contexte des plugins communautaires disponibles, et te suggérer les meilleures solutions.
```bash
skills_pal recom
```

### 3. Scan de la Dette Technique
Analyse le code source de ton projet pour trouver la dette technique explicite (commentaires `TODO`, `FIXME`) et les avertissements de compilation.
```bash
skills_pal scan
```

### 4. Mise à Jour Automatique
Télécharge et installe automatiquement la dernière version de Skills Pal depuis Github.
```bash
skills_pal update
```

---

## 🌍 Architecture & Déploiement (Pour les contributeurs)

Ce dépôt contient deux binaires distincts :

- **Le Client CLI** (`skills_pal`) : `src/main.rs`
- **Le Serveur API** (`server`) : `src/server/main.rs`

### Déployer son propre Serveur (Railway)

Si tu souhaites héberger ta propre instance du registre de plugins :
1. Connecte ton compte Railway à ton fork de ce dépôt GitHub.
2. Provisionne une base de données **PostgreSQL**.
3. Dans les variables d'environnement de ton service web, ajoute : `DATABASE_URL=postgresql://...`
4. Dans **Settings > Deploy** de Railway :
   - **Custom Build Command** : `cargo build --release --bin server && cp target/release/server ./server`
   - **Custom Start Command** : `./server`

Le serveur construira automatiquement les tables SQL, lancera son cache en RAM (0 latence), et commencera à scraper GitHub en arrière-plan.

---

## 🧩 Créer un Plugin pour Skills Pal

Tu as développé un outil ou un script génial et tu veux que Skills Pal le recommande aux autres développeurs ?
Rien de plus simple :

1. Crée un dépôt public sur GitHub.
2. Ajoute le topic (tag) **`skills-pal-plugin`** dans la description de ton dépôt (bouton ⚙️ en haut à droite).
3. Le serveur backend de Skills Pal scannera GitHub et l'ajoutera automatiquement à son registre public sous 12h !

---

## 🔒 Sécurité & Confidentialité

- **Audit complet validé** : Protection intégrale contre les injections SQL (via Parameter Binding).
- **Rate-Limiting Global** : Le serveur utilise `tower::limit::GlobalConcurrencyLimitLayer` pour prévenir les attaques DDoS et la saturation de la DB.
- **Transparence** : Ton code source n'est envoyé qu'à l'API LLM de ton choix (OpenAI/Mistral) sans intermédiaire obscur. Les clés API sont stockées uniquement en local (`.skillspal.toml` est `.gitignore`).

---

<div align="center">
  <i>Construit avec passion, Rust, et beaucoup de café ☕</i>
</div>
