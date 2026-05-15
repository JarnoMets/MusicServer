# MusicServer AI Agent Instructions

This guide helps AI agents understand the specialized architecture and workflows of the MusicServer project.

## 🛠 Build & Development

| Task | Backend (Rust) | Frontend (Vue/TS) |
| :--- | :--- | :--- |
| **Check** | `cargo check` | `npm run lint` |
| **Build** | `cargo build` | `npm run build` |
| **Test** | `cargo test` | `npm run test` |
| **Run** | `cargo run` | `npm run dev` |

- **Migrations**: ⚠️ Do NOT create `.sql` migration files. Schema changes are handled programmatically in [backend/src/db/schema.rs](backend/src/db/schema.rs).
- **Environment**: Backend expects `DATABASE_URL`, `JWT_SECRET`, and `DISCOGS_TOKEN`.

## 🏗 Architecture & Conventions

### Backend (Rust / Actix-Web)
- **Database**: PostgreSQL with `sqlx`. Queries are often written in dedicated services within [backend/src/services/](backend/src/services/).
- **Models**: State and domain entities reside in [backend/src/models/](backend/src/models/).
- **Error Handling**: Use `actix_web::HttpResponse` for route returns. Check [backend/src/routes.rs](backend/src/routes.rs) for patterns.
- **Audio Processing**: High-performance tasks use `symphonia` and `aubio`.

### Frontend (Vue 3 / Pinia)
- **State Management**: Global state uses Pinia stores in [frontend/src/stores/](frontend/src/stores/).
- **Component Pattern**: Feature-based organization in [frontend/src/features/](frontend/src/features/).
- **Type Safety**: Ensure [frontend/src/types/](frontend/src/types/) are synchronized with backend models.

## 🔄 Core Workflows

- **Audio Streaming**: Uses `SSE` (Server-Sent Events) for real-time updates. See [backend/src/services/cache_service.rs](backend/src/services/cache_service.rs).
- **Music Sync**: The `file_sync_service.rs` manages the ingestion of new files into the database.
- **Git**: Always branch off `dev`. Follow [copilot-instructions.md](copilot-instructions.md) for conventional commits.

## ⚠️ Common Pitfalls
- **Programmatic Schema**: If adding a column, update [backend/src/db/schema.rs](backend/src/db/schema.rs).
- **Circular Dependencies**: Be careful with imports between Pinia stores and Vue composables.
- **Audio Headers**: Audio elements in the frontend use token-based query parameters for auth since they can't set custom headers easily.
