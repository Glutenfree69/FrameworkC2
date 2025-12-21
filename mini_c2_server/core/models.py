"""
SQLAlchemy models - Tables agents et tasks
"""
from datetime import datetime, timezone
from typing import Optional

from sqlalchemy import String, Integer, DateTime, ForeignKey, Text
from sqlalchemy.orm import Mapped, mapped_column, relationship

from core.database import Base


def utc_now() -> datetime:
    """Return current UTC time as timezone-aware datetime."""
    return datetime.now(timezone.utc)


class Agent(Base):
    """Table des agents enregistrés."""
    
    __tablename__ = "agents"
    
    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    username: Mapped[str] = mapped_column(String(255))
    hostname: Mapped[str] = mapped_column(String(255))
    internal_ip: Mapped[str] = mapped_column(String(45))
    os_version: Mapped[str] = mapped_column(String(255))
    first_seen: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=utc_now
    )
    last_seen: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=utc_now, onupdate=utc_now
    )
    
    # Relation vers les tâches
    tasks: Mapped[list["Task"]] = relationship(
        "Task", back_populates="agent", cascade="all, delete-orphan"
    )
    
    def __repr__(self) -> str:
        return f"<Agent {self.hostname} ({self.id[:8]}...)>"


class Task(Base):
    """Table des tâches (commandes) pour les agents."""
    
    __tablename__ = "tasks"
    
    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    agent_id: Mapped[str] = mapped_column(
        String(36), ForeignKey("agents.id", ondelete="CASCADE")
    )
    command: Mapped[str] = mapped_column(Text)
    status: Mapped[str] = mapped_column(
        String(20), default="pending"
    )  # pending, sent, completed, failed
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), default=utc_now
    )
    sent_at: Mapped[Optional[datetime]] = mapped_column(
        DateTime(timezone=True), nullable=True
    )
    completed_at: Mapped[Optional[datetime]] = mapped_column(
        DateTime(timezone=True), nullable=True
    )
    result: Mapped[Optional[str]] = mapped_column(Text, nullable=True)
    
    # Relation vers l'agent
    agent: Mapped["Agent"] = relationship("Agent", back_populates="tasks")
    
    def __repr__(self) -> str:
        return f"<Task {self.id}: {self.command[:20]}... ({self.status})>"
