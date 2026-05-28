## 1. Project Infrastructure

- [ ] 1.1 Create `package.json` with `typescript` and `esbuild` dependencies.
- [ ] 1.2 Create `tsconfig.json` with strict typing and modern target.
- [ ] 1.3 Add `server/public/app.js` to `.gitignore`.
- [ ] 1.4 Update `flake.nix` or `shell.nix` to include Node.js if not present.

## 2. Source Conversion

- [ ] 2.1 Move `server/public/app.js` to `server/public/src/app.ts`.
- [ ] 2.2 Define core interfaces: `House`, `Tenant`, `Device`, `Telemetry`.
- [ ] 2.3 Add type annotations to global variables and function parameters.
- [ ] 2.4 Fix all initial type errors (casting DOM elements, refining types).
- [ ] 2.5 Ensure all `window.*` assignments are properly handled via `interface Window`.

## 3. Build & Integration

- [ ] 3.1 Implement build script in `package.json` using `esbuild`.
- [ ] 3.2 Update `dev.sh` to run `esbuild --watch` in the background.
- [ ] 3.3 Verify that the compiled `app.js` is correctly served and functional.

## 4. Verification

- [ ] 4.1 Run full type check with `tsc --noEmit`.
- [ ] 4.2 Verify Admin UI functionality (Houses, Tenants, Devices) in the browser.
- [ ] 4.3 Verify Overview and History tabs.
