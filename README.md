# 🎯 Mini C2 Framework

Bienvenue dans le projet **Mini C2**, un framework Command & Control minimaliste et éducatif.

Ce projet est conçu pour apprendre les concepts fondamentaux du développement de malwares (C2 Architecture) et de la sécurité offensive, tout en appliquant des standards de développement Python rigoureux (Typage strict, Architecture propre).

## 🏗 Architecture

Le projet est divisé en deux composants principaux :

1.  **Server (`mini_c2_server`)** :
    *   Le "Cerveau" de l'opération.
    *   API REST avec FastAPI.
    *   Base de données SQLite (via SQLModel) pour la persistance des agents et des tâches.
    *   Permet aux opérateurs d'envoyer des commandes.

2.  **Agent (`mini_c2_agent`)** :
    *   L'"Implant" déployé sur la cible.
    *   Polling HTTP (Beaconing) pour récupérer les tâches.
    *   Exécution de commandes système (`subprocess`).
    *   Renvoi des résultats au serveur.

## 🚀 Démarrage Rapide

### Prérequis

*   Python 3.11+
*   `uv` (recommandé) ou `pip`

### 1. Démarrer le Serveur

```bash
cd mini_c2_server
pip install -r requirements.txt  # ou `uv sync`
uvicorn main:app --reload
```

Le serveur écoute sur `http://127.0.0.1:8000`.

### 2. Démarrer l'Agent

Dans un autre terminal :

```bash
cd mini_c2_agent
pip install -r requirements.txt # ou `uv sync`
python main.py
```

L'agent va s'enregistrer et commencer à demander des instructions.

## 📚 Documentation

Pour plus de détails, consultez les README spécifiques dans chaque dossier :

*   [Server README](mini_c2_server/README.md)
*   [Agent README](mini_c2_agent/README.md)

## ⚠️ Avertissement

Ce code est fourni à des fins **éducatives uniquement**. L'utilisation de ce logiciel pour attaquer des cibles sans consentement écrit préalable est illégale. Les développeurs déclinent toute responsabilité en cas de mauvaise utilisation.
