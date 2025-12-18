# AGENTS.md - Project Guidelines & Context

This file serves as the primary instruction set for AI agents (specifically Jules) working on the **Mini C2** project.

## 🎯 Project Philosophy

1.  **Educational First**: The primary goal is to learn Network/C2 concepts and Python deeply.
    - Code should be clear, well-structured, and idiomatic.
    - Complex logic should be explained in comments or documentation.
    - Avoid "magic" code; prefer explicit implementations that demonstrate the underlying mechanics.
2.  **Research-Driven**:
    - **Benchmark**: Before implementing a feature, ALWAYS research how it is handled in established frameworks (e.g., **Sliver**, **Cobalt Strike**, **Mythic**, **Havoc**).
    - **Adapt**: Implement a simplified but architecturally accurate version of these standard mechanisms.
3.  **Incremental & Flexible**:
    - Propose meaningful changes.
    - **Batching**: You may implement multiple small fixes or refactors in a single session if they are low-risk and related.
    - For major features, stick to one per session to keep it digestible.
4.  **Strict Quality**:
    - **Type Safety**: Use `mypy` strict standards. No `Any` without a very good reason.
    - **Linting**: Follow `ruff`/PEP 8 standards.
    - **Documentation**: Keep READMEs and docstrings up to date.

## 🛠 Technical Stack

- **Server**: Python 3.11+, FastAPI, Pydantic, Uvicorn.
- **Agent**: Python 3.11+, Requests, Standard Library (subprocess, socket, etc.).
- **Database**: SQLModel (SQLite).
- **Tools**: `uv` for dependency management.

## 🤖 Directives for AI Agents

### When Refactoring
- **Hunt Dead Code**: Aggressively identify and remove unused imports, variables, functions, or files.
- **Explain "Why"**: When replacing a component (e.g., list -> database), explain the benefits (persistence, concurrency) and the trade-offs.
- **Verify**: Never assume a refactor works. Verify that the Agent and Server still communicate correctly.

### When Implementing Features
- **Plan First**: Always analyze the current state before coding.
- **Research**: Use the `google_search` tool (or similar) to understand how Sliver/Cobalt Strike implements the feature.

## 🗺 Roadmap

1.  **Persistence**: Move from in-memory lists to SQLite/SQLModel.
2.  **Real Execution**: Agent executes shell commands (`subprocess`) and captures output.
3.  **Operator Interface**: CLI or TUI to manage agents/tasks.
4.  **Advanced Comms**: New protocols (DNS, Custom TCP) and encryption.
