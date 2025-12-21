"""
CRUD operations - Fonctions async pour manipuler la DB
"""
from datetime import datetime, timezone
from typing import Optional

from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession

from core.models import Agent, Task


def utc_now() -> datetime:
    """Return current UTC time as timezone-aware datetime."""
    return datetime.now(timezone.utc)


async def create_or_update_agent(
    db: AsyncSession,
    agent_id: str,
    username: str,
    hostname: str,
    internal_ip: str,
    os_version: str,
) -> Agent:
    """
    Enregistre un nouvel agent ou met à jour last_seen si existant.
    
    Returns:
        Agent: L'instance agent créée ou mise à jour.
    """
    result = await db.execute(select(Agent).where(Agent.id == agent_id))
    agent = result.scalar_one_or_none()
    
    if agent:
        # Agent existe -> update last_seen
        agent.last_seen = utc_now()
        agent.username = username
        agent.hostname = hostname
        agent.internal_ip = internal_ip
        agent.os_version = os_version
    else:
        # Nouvel agent -> insert
        agent = Agent(
            id=agent_id,
            username=username,
            hostname=hostname,
            internal_ip=internal_ip,
            os_version=os_version,
        )
        db.add(agent)
    
    await db.commit()
    await db.refresh(agent)
    return agent


async def get_agent(db: AsyncSession, agent_id: str) -> Optional[Agent]:
    """Récupère un agent par son ID."""
    result = await db.execute(select(Agent).where(Agent.id == agent_id))
    return result.scalar_one_or_none()


async def get_all_agents(db: AsyncSession) -> list[Agent]:
    """Récupère tous les agents enregistrés."""
    result = await db.execute(select(Agent).order_by(Agent.last_seen.desc()))
    return list(result.scalars().all())


async def create_task(
    db: AsyncSession,
    agent_id: str,
    command: str,
) -> Task:
    """
    Crée une nouvelle tâche pour un agent.
    
    Returns:
        Task: La tâche créée.
    
    Raises:
        ValueError: Si l'agent n'existe pas.
    """
    # Vérifier que l'agent existe
    agent = await get_agent(db, agent_id)
    if not agent:
        raise ValueError(f"Agent {agent_id} not found")
    
    task = Task(
        agent_id=agent_id,
        command=command,
        status="pending",
    )
    db.add(task)
    await db.commit()
    await db.refresh(task)
    return task


async def get_pending_task(db: AsyncSession, agent_id: str) -> Optional[Task]:
    """
    Récupère la plus ancienne tâche pending pour un agent et la marque comme 'sent'.
    
    Returns:
        Task | None: La tâche si trouvée, None sinon.
    """
    result = await db.execute(
        select(Task)
        .where(Task.agent_id == agent_id, Task.status == "pending")
        .order_by(Task.created_at.asc())
        .limit(1)
    )
    task = result.scalar_one_or_none()
    
    if task:
        task.status = "sent"
        task.sent_at = utc_now()
        await db.commit()
        await db.refresh(task)
    
    return task


async def get_task_queue_position(db: AsyncSession, agent_id: str) -> int:
    """Compte le nombre de tâches pending pour un agent."""
    result = await db.execute(
        select(Task).where(Task.agent_id == agent_id, Task.status == "pending")
    )
    return len(list(result.scalars().all()))
