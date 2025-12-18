# 🧠 Mini C2 Server

Ce projet est un serveur de **Command & Control (C2)** minimaliste développé en Python avec **FastAPI**. Il sert de "cerveau" central pour gérer des agents distants, recevoir leurs enregistrements et leur distribuer des tâches via HTTP.

> **Note :** Ce projet est à but strictement éducatif pour comprendre les mécanismes de communication C2 (Beaconing, Polling) et le typage strict en Python.

---

## ⚡ Fonctionnalités

- **Check-in** : Enregistrement des nouveaux agents (UUID, IP, User, OS)
- **Polling** : Les agents viennent récupérer leurs tâches périodiquement ("Beaconing")
- **Queueing** : Système de file d'attente FIFO (First-In-First-Out) pour les commandes
- **Strict Typing** : Utilisation intensive de Pydantic pour garantir l'intégrité et la validation des données échangées

---

## 🛠️ Installation

Ce projet utilise **uv** pour la gestion moderne des dépendances Python.

1. Assurez-vous d'être dans le dossier du projet :

```bash
cd mini_c2_server
```

2. Installez les dépendances :

```bash
uv sync
```

---

## 🚀 Lancement

Pour démarrer le serveur en mode développement (avec rechargement automatique à la modification des fichiers) :

```bash
uv run uvicorn main:app --reload
```

- **Serveur** : `http://127.0.0.1:8000`
- Les logs d'accès s'affichent directement dans le terminal

---

## 📖 Documentation API

Grâce à FastAPI, une documentation interactive est générée automatiquement. Une fois le serveur lancé, accédez à :

| Interface | URL |
|-----------|-----|
| **Swagger UI** (Test des routes) | http://127.0.0.1:8000/docs |
| **ReDoc** (Lecture seule) | http://127.0.0.1:8000/redoc |

---

## 🎮 Guide Opérateur (Admin)

Puisque nous n'avons pas encore d'interface graphique (GUI), vous pouvez envoyer des commandes aux agents via l'API.

### 1. Attendre un Agent

Lancez le serveur, puis lancez l'agent. Récupérez l'ID de l'agent qui s'affiche dans les logs du serveur :

```
🔔 CHECK-IN: DESKTOP-XYZ (550e8400-e29b-41d4-a716-446655440000)
```

### 2. Envoyer une commande (Tasking)

Utilisez `curl` ou l'interface Swagger pour ajouter une tâche dans la file d'attente de l'agent.

**Exemple avec curl :**

```bash
curl -X POST "http://127.0.0.1:8000/api/v1/admin/tasks" \
     -H "Content-Type: application/json" \
     -d '{"agent_id": "COLLER_UUID_ICI", "command": "whoami"}'
```

L'agent récupérera cette commande lors de son prochain "réveil" (Beacon).

---

## 📂 Structure du Projet

```
mini_c2_server/
├── main.py          # Point d'entrée, logique des routes et stockage mémoire
├── pyproject.toml   # Configuration du projet et dépendances (géré par uv)
├── .venv/           # Environnement virtuel (géré par uv)
└── api/
    ├── __init__.py
    └── schemas.py   # Modèles Pydantic (le contrat de données strict)
```
