# Bitcoin Kernel Binding Conformance Tests

This directory contains a language-agnostic conformance testing framework for Bitcoin Kernel bindings.

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

1. [**Orchestrator**](./orchestrator): Spawns handler binary, sends test requests, validates responses
2. **Handler Binary**: Implements protocol, calls binding API, returns results
   - [Go handler](./go-handler) for the [Go binding](https://github.com/stringintech/go-bitcoinkernel)
   - [Rust handler](./rust-handler) for the [Rust binding](https://github.com/TheCharlatan/rust-bitcoinkernel)
3. [**Test Cases**](./testdata): JSON files defining requests and expected responses

## Getting Started

### Cloning

Clone the repository with submodules:

```bash
git clone --recurse-submodules https://github.com/stringintech/kernel-bindings-spec.git
```
**Note:** go-handler currently depends on go-bitcoinkernel via a submodule.


### Building

Use the provided Makefile to build the project:

```bash
# Build everything!
make build
```

### Running Tests

```bash
# Run all conformance tests with both Go and Rust handlers
make test

# Run a specific test file with both Go and Rust handlers
make test-single TEST=testdata/chainstate_basic.json
```