# 🕵️ Mini C2 Agent

Cet agent est la partie "implant" du projet Mini C2. Il est conçu pour être exécuté sur la machine cible. Il communique avec le serveur C2 pour recevoir des instructions et renvoyer les résultats.

> **Note :** Ce projet est à but strictement éducatif. N'exécutez cet agent que sur des systèmes que vous possédez ou pour lesquels vous avez une autorisation explicite.

---

## ⚡ Fonctionnalités

- **Beaconing** : L'agent contacte le serveur à intervalles réguliers (toutes les 5 secondes par défaut).
- **Exécution de commandes** : Exécute des commandes shell via `subprocess`.
- **Rapport** : Renvoie la sortie (stdout/stderr) des commandes au serveur.
- **Identification** : Collecte des informations de base sur le système (Hostname, IP, User, OS).

---

## 🛠️ Installation

1. Assurez-vous d'avoir Python 3.11+ installé.
2. Placez-vous dans le dossier de l'agent :

```bash
cd mini_c2_agent
```

3. Installez les dépendances :

```bash
uv sync
```
ou
```bash
pip install requests pydantic
```

---

## 🚀 Lancement

Pour démarrer l'agent :

```bash
python main.py
```

Assurez-vous que le serveur (`mini_c2_server`) est en cours d'exécution sur `http://127.0.0.1:8000`.

L'agent va :
1. Collecter les infos système.
2. S'enregistrer auprès du serveur ("Check-in").
3. Entrer dans une boucle infinie pour demander du travail.

---

## ⚙️ Configuration

Actuellement, la configuration est définie en haut du fichier `main.py` :

```python
SERVER_URL = "http://127.0.0.1:8000"
SLEEP_TIME = 5
```

Vous pouvez modifier ces valeurs pour changer l'adresse du serveur ou la fréquence de beaconing.
