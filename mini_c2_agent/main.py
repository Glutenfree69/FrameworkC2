import platform
import socket
import os
import uuid
import time
import requests # type: ignore
from pydantic import BaseModel
from typing import Optional

# --- CONFIGURATION ---
SERVER_URL = "http://127.0.0.1:8000"
SLEEP_TIME = 5

# --- LE CONTRAT (Copié du serveur pour être strict) ---

class AgentCheckIn(BaseModel):
    agent_id: str
    username: str
    hostname: str
    internal_ip: str
    os_version: str

class TaskResponse(BaseModel):
    command: Optional[str] = None

# --- FONCTIONS ---

def get_system_info() -> AgentCheckIn:
    """
    Récupère les infos et retourne un OBJET strict.
    """
    try:
        user = os.getlogin()
    except:
        user = "unknown"

    # On construit l'objet Pydantic directement
    return AgentCheckIn(
        agent_id=str(uuid.uuid4()),
        username=user,
        hostname=socket.gethostname(),
        internal_ip="127.0.0.1", # Simplifié
        os_version=f"{platform.system()} {platform.release()}"
    )

def main():
    # 1. Collecte (Type strict : AgentCheckIn)
    my_info = get_system_info()
    
    print(f"🕵️  AGENT DÉMARRÉ - ID: {my_info.agent_id}")

    # 2. Check-in
    try:
        # requests.post attend un dict ou du json, 
        # Pydantic a une méthode géniale pour ça : .model_dump()
        response = requests.post(
            f"{SERVER_URL}/api/v1/checkin", 
            json=my_info.model_dump()
        )
        response.raise_for_status() # Lève une erreur si code != 200
        print("✅ Enregistrement validé par le serveur.")
        
    except Exception as e:
        print(f"❌ Echec du Check-in : {e}")
        return

    # 3. Boucle de travail
    while True:
        try:
            print("💤 Demande de travail...")
            resp = requests.get(f"{SERVER_URL}/api/v1/tasks/{my_info.agent_id}")

            # On force la réponse à rentrer dans notre modèle TaskResponse
            task_data = TaskResponse.model_validate(resp.json())
            
            if task_data.command:
                print(f"⚙️  ORDRE REÇU ET VALIDÉ : {task_data.command}")
                # Exécution (simulation)
            else:
                print("ø Rien à faire.")

        except Exception as e:
            print(f"⚠️ Erreur réseau ou format invalide : {e}")

        time.sleep(SLEEP_TIME)

if __name__ == "__main__":
    main()
 