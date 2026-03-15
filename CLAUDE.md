# CLAUDE.md

## Overview

This directory contains the workspace manifests, toolchain pin, and repo-level documentation for 3D Chess.

## Index

| File | Contents (WHAT) | Read When (WHEN) |
| --- | --- | --- |
| `Cargo.toml` | Workspace members, shared dependencies, and lint policy | Adding crates, changing shared versions, or tightening workspace rules |
| `Cargo.lock` | Locked dependency graph for the Rust workspace | Reviewing dependency resolution or checking whether dependency changes were captured |
| `README.md` | Repo-level milestone status, architecture boundaries, and developer commands | Orienting to the project or confirming shipped scope before editing crate code |
| `rust-toolchain.toml` | Pinned Rust toolchain for local and CI builds | Aligning compiler versions or debugging toolchain drift |
