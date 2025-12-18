from pydantic import BaseModel
from typing import Optional # Pour dire que quelque chose peut être "None"

# --- REQUÊTES (Ce que l'on reçoit) ---

class AgentCheckIn(BaseModel):
    agent_id: str
    username: str
    hostname: str
    internal_ip: str
    os_version: str

class TaskRequest(BaseModel):
    agent_id: str
    command: str

# --- RÉPONSES (Ce que l'on renvoie) ---

class CheckInResponse(BaseModel):
    status: str
    message: str

class TaskResponse(BaseModel):
    # Optional[str] veut dire : soit un texte, soit None (rien)
    command: Optional[str] = None 

class AdminTaskResponse(BaseModel):
    status: str
    position: int # Ici on est strict : ça DOIT être un nombre entier
