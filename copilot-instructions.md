# MusicServer Project Instructions

## Git Workflow
Follow these steps for any feature or fix:
1. **Branching**: Always branch off from `dev`. Use naming conventions like `feat/feature-name` or `fix/issue-description`.
2. **Atomic Commits**: Group related changes into a single commit. Avoid "work in progress" commits if they break the build.
3. **Commit Messages**: Follow Conventional Commits (e.g., `feat: add lyrics service`, `fix: handle discogs timeout`).
4. **Validation**: Before merging or finalizing a task, ensure:
   - Backend compiles: Run `cargo check` or `cargo build` in `backend/`.
   - Frontend compiles: Run `npm run build` or `npm run lint` in `frontend/`.
5. **Merging**: Merge back into `dev` after validation.
6. **Documentation**: Update the [README.md](README.md) or specific service logs if public APIs or database schemas change.

## Coding Standards
- **Errors**: In the Rust backend, use the project's custom error results (check `backend/src/lib.rs` for `AppError` or similar).
- **Consistency**: Ensure TypeScript interfaces in `frontend/src/types` match Rust models in `backend/src/models/`.
- **Services**: Always check `backend/src/services/` before implementing new business logic to avoid duplication.
