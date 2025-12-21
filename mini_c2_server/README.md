# 🧠 Mini C2 Server

Ce projet est un serveur de **Command & Control (C2)** minimaliste développé en Python avec **FastAPI**. Il sert de "cerveau" central pour gérer des agents distants, recevoir leurs enregistrements et leur distribuer des tâches via HTTP.

> **Note :** Ce projet est à but strictement éducatif pour comprendre les mécanismes de communication C2 (Beaconing, Polling) et le typage strict en Python.

---

## ⚡ Fonctionnalités

- **Check-in** : Enregistrement des nouveaux agents (UUID, IP, User, OS)
- **Polling** : Les agents viennent récupérer leurs tâches périodiquement ("Beaconing")
- **Tasking** : Envoi de commandes aux agents via l'API admin
- **Exécution** : Les agents exécutent les commandes et renvoient les résultats
- **Queueing** : Système de file d'attente FIFO (First-In-First-Out) pour les commandes
- **Strict Typing** : Utilisation intensive de Pydantic pour garantir l'intégrité des données
- **Persistance SQLite** : Base de données locale pour conserver agents, tâches et résultats

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
- **Base de données** : `c2.db` (créée automatiquement au premier lancement)
- Les logs d'accès s'affichent directement dans le terminal

---

## 📖 Documentation API

Grâce à FastAPI, une documentation interactive est générée automatiquement. Une fois le serveur lancé, accédez à :

| Interface | URL |
|-----------|-----|
| **Swagger UI** (Test des routes) | http://127.0.0.1:8000/docs |
| **ReDoc** (Lecture seule) | http://127.0.0.1:8000/redoc |

---

## 🗄️ Base de Données

Le serveur utilise **SQLite** avec **SQLAlchemy async** pour persister les données.

### Tables

| Table | Description |
|-------|-------------|
| `agents` | Agents enregistrés (id, hostname, username, IP, OS, first_seen, last_seen) |
| `tasks` | Commandes par agent (command, status, result, timestamps) |

### Statuts des tâches

| Status | Description |
|--------|-------------|
| `pending` | En attente d'être récupérée par l'agent |
| `sent` | Envoyée à l'agent, en cours d'exécution |
| `completed` | Exécutée avec succès, résultat disponible |
| `failed` | Échec d'exécution |

### Inspecter la DB

```bash
# Lister les tables
sqlite3 c2.db ".tables"

# Voir les agents enregistrés
sqlite3 c2.db "SELECT id, hostname, username, last_seen FROM agents;"

# Voir les tâches avec leurs résultats
sqlite3 c2.db "SELECT id, command, status, result FROM tasks;"

# Voir uniquement les tâches complétées
sqlite3 c2.db "SELECT command, result FROM tasks WHERE status='completed';"

# Supprimer la DB pour repartir de zéro
rm c2.db
```

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

### 3. Voir les résultats

Une fois la commande exécutée par l'agent, le résultat est stocké en base :

```bash
# Voir le résultat de la dernière tâche
sqlite3 c2.db "SELECT command, status, result FROM tasks ORDER BY id DESC LIMIT 1;"

# Ou via l'API (Swagger UI)
# GET http://127.0.0.1:8000/docs
```

---

## 🔌 Endpoints API

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| `POST` | `/api/v1/checkin` | Enregistrement d'un agent |
| `GET` | `/api/v1/tasks/{agent_id}` | Agent récupère sa prochaine tâche |
| `POST` | `/api/v1/tasks/result` | Agent envoie le résultat d'une tâche |
| `POST` | `/api/v1/admin/tasks` | Admin ajoute une tâche pour un agent |

---

## 📂 Structure du Projet

```
mini_c2_server/
├── main.py              # Point d'entrée FastAPI + routes
├── pyproject.toml       # Dépendances (FastAPI, SQLAlchemy, aiosqlite)
├── c2.db                # Base SQLite (générée au runtime)
├── api/
│   ├── __init__.py
│   └── schemas.py       # Modèles Pydantic (validation API)
└── core/
    ├── __init__.py
    ├── database.py      # Configuration SQLAlchemy async
    ├── models.py        # Modèles ORM (tables Agent, Task)
    └── crud.py          # Opérations CRUD async
```
