---
name: project-versioning
description: Automates the version bump, testing, and release process for the lmha3 project. Use this when a new version needs to be released.
---

# Project Versioning Workflow

This skill automates the standard release process for the `lmha3` project.

## Workflow Steps

1. **Identify the New Version**: Determine the next version number (e.g., bumping from 0.0.7 to 0.0.8).
2. **Update Cargo.toml Files**:
   - Update `version` in `lmha-core/Cargo.toml`.
   - Update `version` in `server/Cargo.toml`.
   - Update `version` in `lmha-admin/Cargo.toml`.
3. **Update Nix Configuration**:
   - Update `version` in `default.nix`.
4. **Update Lockfile and Verify**:
   - Run `cargo build` to update `Cargo.lock`.
   - Run `cargo test` to ensure all tests pass.
5. **Commit and Tag**:
   - Stage changes: `git add .`
   - Commit: `git commit -m "chore: bump version to X.Y.Z"`
   - Tag: `git tag vX.Y.Z`
6. **Push to origin (named lluki)**:
   - Push branch: `git push lluki main`
   - Push tag: `git push lluki vX.Y.Z`
7. **Deployment**:
   - **Mandatory Query**: Query the user if they want to deploy the new version directly. Do NOT perform deployment steps automatically.
   - If the user confirms, provide these manual steps:
     1. SSH to the production server: `ssh lisa`
     2. Edit the configuration with sudo: `sudo $EDITOR /etc/nixos/nixos-config/configuration.nix`
     3. Replace the old version tag with the new tag `vX.Y.Z`.
     4. Apply the configuration: `sudo nixos-rebuild switch`

## Reference Commands

- **Build/Lock**: `cargo build`
- **Test**: `cargo test`
- **Git Push (to lluki)**: `git push lluki main && git push lluki v<version>`
- **SSH to lisa**: `ssh lisa`

