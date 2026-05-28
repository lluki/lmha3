## Context

The LMHA frontend is currently a single `app.js` file served directly by the backend. While simple, this approach lacks the safety nets of a modern development environment. We need to introduce a compilation step that preserves this simplicity while adding type safety.

## Goals / Non-Goals

**Goals:**
- Migrate the existing `app.js` to TypeScript without breaking existing functionality.
- Introduce a high-performance build step using `esbuild`.
- Define clear interfaces for all API entities.
- Integrate type checking into the development workflow.

**Non-Goals:**
- Porting to a framework (React, Vue, etc.) - we stay with Vanilla DOM.
- Splitting the app into dozens of small files (for now, we'll keep it relatively flat).

## Decisions

- **Build Tool**: **esbuild**. It is extremely fast and requires minimal configuration, making it ideal for a project that wants to stay "close to the metal".
- **Source Structure**: Move source to `server/public/src/` (e.g., `app.ts`). The build tool will output to the existing `server/public/app.js` (which we should add to `.gitignore`).
- **TS Configuration**: Strict mode enabled. We want to catch as many potential issues as possible.
- **Dependency Management**: Use `npm` with a `package.json` in the project root to manage `typescript` and `esbuild` as dev dependencies.

## Risks / Trade-offs

- [Risk] Build step adds friction to development. → [Mitigation] Use `esbuild --watch` in `dev.sh` to make the build instantaneous and automatic.
- [Risk] Source maps/Debugging complexity. → [Mitigation] Configure `esbuild` to generate inline source maps for easy browser debugging.
- [Risk] Initial migration effort for types. → [Mitigation] Start with some `any` types where absolutely necessary, but prioritize strict interfaces for the main API models.
