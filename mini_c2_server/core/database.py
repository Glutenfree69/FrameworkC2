"""
Database configuration - SQLite async avec SQLAlchemy 2.0
"""
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import DeclarativeBase
from typing import AsyncGenerator

# SQLite async - fichier c2.db à la racine du serveur
DATABASE_URL = "sqlite+aiosqlite:///c2.db"

# Engine async - echo=True pour debug SQL (désactiver en prod)
engine = create_async_engine(DATABASE_URL, echo=False)

# Session factory async
async_session = async_sessionmaker(
    engine,
    class_=AsyncSession,
    expire_on_commit=False,
)


class Base(DeclarativeBase):
    """Base class for all SQLAlchemy models."""
    pass


async def get_db() -> AsyncGenerator[AsyncSession, None]:
    """
    Dependency injection pour FastAPI.
    Fournit une session DB et la ferme automatiquement après la requête.
    """
    async with async_session() as session:
        try:
            yield session
        finally:
            await session.close()


async def init_db() -> None:
    """
    Crée toutes les tables au démarrage.
    À appeler dans le lifespan de FastAPI.
    """
    async with engine.begin() as conn:
        from core.models import Base  # Import local pour éviter circular import
        await conn.run_sync(Base.metadata.create_all)
