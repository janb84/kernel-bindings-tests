# Bitcoin Kernel Binding Conformance Tests

This repository contains a language-agnostic conformance testing framework for Bitcoin Kernel bindings.

## ⚠️ Work in Progress

## Overview

The framework ensures that all language bindings (Go, Python, Rust, etc.) behave identically by:
- Defining a standard JSON protocol for testing
- Providing shared test cases that work across all bindings
- Enforcing consistent error handling and categorization

## Architecture

```
┌─────────────┐         ┌──────────────────┐
│ Orchestrator│────────▶│  Handler Binary  │
│  (Go Test   │ stdin   │  (Go/Rust/etc)   │
│   Runner)   │◀────────│                  │
└─────────────┘ stdout  └──────────────────┘
       │                         │
       │                         │
       ▼                         ▼
  ┌─────────┐            ┌────────────────┐
  │ Test    │            │ Binding API    │
  │ Cases   │            └────────────────┘
  │ (JSON)  │            
  └─────────┘
```

**This repository contains:**
1. [**Orchestrator**](./orchestrator): Spawns handler binary, sends test requests via stdin, validates responses from stdout
2. [**Test Cases**](./testdata): JSON files defining requests and expected responses

**Handler binaries** are not hosted in this repository. They must be implemented separately and should:
- Implement the JSON protocol for communication with the orchestrator
- Call the binding API to execute operations
- Pin to a specific version/tag of this test repository