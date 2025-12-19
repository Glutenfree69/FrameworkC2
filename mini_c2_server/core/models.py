from typing import Optional
from sqlmodel import Field, SQLModel
from datetime import datetime

class Agent(SQLModel, table=True):
    id: str = Field(primary_key=True)  # Using agent_id as primary key
    hostname: str
    username: str
    internal_ip: str
    os_version: str
    first_seen: datetime = Field(default_factory=datetime.utcnow)
    last_seen: datetime = Field(default_factory=datetime.utcnow)

class Task(SQLModel, table=True):
    id: Optional[int] = Field(default=None, primary_key=True)
    agent_id: str = Field(foreign_key="agent.id")
    command: str
    status: str = Field(default="pending")  # pending, sent, completed, error
    result: Optional[str] = None
    created_at: datetime = Field(default_factory=datetime.utcnow)
    executed_at: Optional[datetime] = None
