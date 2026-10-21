# 0002 — First-milestone stack

**Status:** accepted, 2026-10-19.

## Context and options

The user prefers Rust for the CLI core and local inference with modest resource use. Embedding an inference engine would increase the initial integration and packaging scope; a separate runtime exposes a replaceable local HTTP boundary.

## Decision and approval

The architect proposed Rust for the core, CLI, shared contract, and initial module, with Ollama as a separately installed inference service and Tavily as the initial search backend. The user explicitly approved [plan 0003](../plans/0003-first-milestone.md) with “Alright let's go ahead on the first milestone. Let's rock!”

## Consequences

The first application uses a Rust workspace and keeps capabilities under `modules/*`. It requires local runtime setup and model files for generated requests/replies. Inference is replaceable behind a narrow interface; the application does not install its own runtime or automatically fall back to cloud inference. Model/quantization selection remains dependent on evaluation. Full self-hosted search, richer interfaces, and other capabilities remain future scope. This decision does not authorize commits.
