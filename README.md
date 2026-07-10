<div align="center">
  <h1>👋 Salut, voici Skills Pal ! 🧠</h1>
  <p><strong>Ton compagnon IA personnel pour nettoyer ton code et te faire découvrir les meilleurs outils.</strong></p>
  
  [![Rust](https://img.shields.io/badge/Rust-1.88.0-orange.svg)](https://www.rust-lang.org)
  [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
  [![Railway](https://img.shields.io/badge/Railway-Deployed-purple.svg)](https://railway.app)
</div>

<br />

## 🌟 C'est quoi au juste ?

Tu sais, ce moment où tu regardes ton code et tu te dis *"il doit y avoir un outil pour faire ça mieux que moi"* ? C'est exactement pour ça qu'on a créé **Skills Pal**.

C'est un petit outil de ligne de commande (CLI) ultra-rapide qui vient scanner ton projet (peu importe le langage). Il lit tes fichiers, discute avec une intelligence artificielle (OpenAI ou Mistral, au choix !), et te recommande les plugins ou outils parfaits pour améliorer ton code ou te débarrasser de ta dette technique.

Et le meilleur ? Il est connecté à une base de données communautaire (hébergée sur Railway) qui se met à jour toute seule en fouillant Github !

---

## 🚀 Comment l'installer en 1 seconde ?

Pas besoin d'installer Rust, Node.js ou de cloner le projet. On a rendu ça super simple. Ouvre ton terminal et colle cette ligne :

**🍎 Sur Mac ou Linux :**
```bash
curl -fsSL https://raw.githubusercontent.com/Julien-Bui/skills_pal/main/install.sh | bash
```

**🪟 Sur Windows (PowerShell) :**
```powershell
iwr https://raw.githubusercontent.com/Julien-Bui/skills_pal/main/install.ps1 -useb | iex
```

Une fois que c'est fait, balade-toi dans n'importe quel dossier de ton ordinateur et lance la commande de bienvenue :
```bash
skills_pal init
```
*(Il te demandera juste ta clé API pour l'IA, et l'URL du serveur si tu en utilises un personnalisé !)*

---

## 🛠️ Les commandes à connaître

L'outil est super simple à utiliser. Tu as trois commandes magiques :

- `skills_pal init` : Crée ton fichier de configuration local. C'est la toute première étape.
- `skills_pal recom` : C'est le cœur du projet ! L'IA regarde dans quoi tu codes (Go, Python, JS, etc.) et te sort les meilleures recommandations du moment.
- `skills_pal scan` : Fait un rapide état des lieux des plugins que tu as déjà.

---

## 🧩 Tu veux proposer ton propre outil ?

C'est un projet communautaire ! Si tu as créé un script génial, un linter, ou n'importe quel outil open-source et que tu veux que notre IA le recommande aux autres développeurs, c'est super facile :

1. Va sur ton dépôt Github public.
2. Ajoute simplement le tag **`skills-pal-plugin`** dans tes topics Github (en haut à droite ⚙️).
3. Notre serveur passe toutes les 12 heures sur Github. Il verra ton dépôt et l'ajoutera automatiquement à la base de données. L'IA commencera à le proposer aux développeurs qui en ont besoin !

---

## 🌍 Pour les curieux (Sous le capot)

Si tu aimes regarder comment les choses sont faites, tu es au bon endroit. Le projet est divisé en deux morceaux, codés avec amour en **Rust** (pour que ce soit ultra rapide et léger) :

1. **Le Client CLI** (`src/main.rs`) : C'est ce qui tourne sur ton ordinateur.
2. **Le Serveur API** (`src/server/main.rs`) : C'est le cerveau central qui liste tous les plugins de la communauté. 
   - Il tourne H24 sur Railway.
   - Il stocke tout dans une base **PostgreSQL** (totalement sécurisée contre les injections).
   - Il garde les données en mémoire vive (RAM) pour répondre en 0 milliseconde.
   - Il se protège tout seul contre les spams avec un système anti-DDoS (Rate Limiter).

Si tu veux contribuer, corriger un bug ou héberger ton propre serveur, tu es plus que le bienvenu ! N'hésite pas à ouvrir une *Issue* ou une *Pull Request*. 

<div align="center">
  <i>Fait pour les développeurs, par des développeurs ☕</i>
</div>
