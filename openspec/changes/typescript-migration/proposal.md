## Why

The current JavaScript-based frontend lacks static type checking, leading to common runtime errors like `ReferenceError` or "cannot read property of undefined". Migrating to TypeScript will introduce compile-time validation, improve developer productivity through better IDE support, and ensure the long-term reliability of the LMHA interface.

## What Changes

- **Build Pipeline**: Introduction of a TypeScript compilation step (using `esbuild` for speed).
- **Language**: Transition from `.js` to `.ts` for all frontend logic.
- **Type Definitions**: Formal definitions for API models (Houses, Tenants, Devices) and DOM elements.
- **Improved Tooling**: Addition of `tsconfig.json` and a `package.json` for frontend dependency management.

## Capabilities

### New Capabilities
- `frontend-type-safety`: Establishes the requirement for all frontend logic to be statically typed and validated against a defined schema before deployment.

### Modified Capabilities
<!-- No functional requirements are changing, only the implementation language and validation process. -->

## Impact

- `server/public/app.js`: Replaced by `server/public/src/app.ts` (or similar structure) and a compiled output.
- `package.json`: New file to manage build dependencies (`typescript`, `esbuild`).
- `tsconfig.json`: New file for TypeScript configuration.
- `dev.sh`: Updated to include the frontend build/watch process.
- `flake.nix` / `shell.nix`: Updated to ensure `node` and `npm` are available in the development environment.
