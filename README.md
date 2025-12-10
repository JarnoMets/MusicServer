# Music Server

A web-based music management and streaming application built with:

- **Backend**: Rust with Actix-web framework
- **Frontend**: Vue 3 with TypeScript and Vite
- **Database**: PostgreSQL
- **Deployment**: Docker + Kubernetes

## Quick Start

### Development

Install dependencies and start the development server:

\`\`\`bash
# Backend + Frontend development servers
./dev.sh

# Or with docker-compose
docker-compose up
\`\`\`

Access the application at:
- Frontend: http://localhost:5173
- Backend API: http://localhost:8081/api

### Docker Deployment

Build and run with Docker Compose:

\`\`\`bash
docker-compose up --build
\`\`\`

Access at http://localhost:3001

### Kubernetes Deployment

Deploy to your k3s cluster:

\`\`\`bash
make deploy
\`\`\`

**Note:** MusicServer shares the PostgreSQL instance from the Card Scorer project (in the `postgres` namespace).

## Project Structure

- `backend/` - Rust Actix-web API
  - `src/` - Source code
  - `Dockerfile` - Container image
- `frontend/` - Vue 3 TypeScript app
  - `src/` - Vue components and logic
  - `Dockerfile` - Nginx container
- `k8s/` - Kubernetes manifests
- `docker-compose.yaml` - Local development compose file

## Available Commands

- \`make dev\` - Run development environment
- \`make build-images\` - Build Docker images
- \`make deploy\` - Deploy to k3s
- \`make clean\` - Clean up k8s resources
- \`./deploy.sh\` - Build and push to Docker Hub

## Environment Variables

### Backend
- \`DATABASE_URL\` - PostgreSQL connection string
- \`RUST_LOG\` - Log level (default: info)

### Frontend
- \`VITE_API_URL\` - Backend API URL (default: http://localhost:8081/api)

## Features

- Playlist management
- Music file management
- RESTful API
- Responsive web UI
- Internationalization (English, Dutch)


## Admin token (rotation & verification)

This service protects admin-only endpoints (genre creation, aliasing, backfills and reprocessing) with a shared secret token. The server does not store the raw token — instead you store the SHA-256 hex of the raw token in the environment variable `ADMIN_TOKEN_SHA256` (recommended: managed as a Kubernetes Secret).

How it works:
- The server prefers the standard `Authorization: Bearer <raw-token>` header. For compatibility the legacy `x-admin-token: <raw-token>` header is also accepted.
- The server computes SHA-256 of the provided raw token and performs a constant-time comparison with the value in `ADMIN_TOKEN_SHA256`.

Rotation steps (Kubernetes):
1. Generate a new raw token and compute its SHA-256 hex:
  ```bash
  # Example: generate a 32-byte random token and its sha256
  RAW_TOKEN=$(head -c 32 /dev/urandom | base64)
  echo $RAW_TOKEN
  echo -n "$RAW_TOKEN" | sha256sum | awk '{print $1}'
  ```
2. Update the k8s Secret (namespace `music`) with the new SHA256 hex:
  ```bash
  kubectl create secret generic music-admin \
    --namespace music \
    --from-literal=ADMIN_TOKEN_SHA256='<sha256-hex>' \
    --dry-run=client -o yaml | kubectl apply -f -
  ```
3. Restart the backend deployment so pods pick up the new env var:
  ```bash
  kubectl rollout restart deployment/backend -n music
  ```

Clients should be updated to use the new raw token. Because the server uses only the SHA256 stored in the secret, you can rotate tokens safely and centrally.

Example curl with Authorization header:
```bash
curl -X POST https://your-backend/api/admin/genres \
  -H "Authorization: Bearer <RAW_TOKEN>" \
  -H "Content-Type: application/json" \
  -d '{"name":"Electronic","description":"Electronic music"}'
```

## Database migrations

The backend automatically creates the database if it does not exist and runs programmatic Rust migrations at startup (see `backend/src/db/schema.rs`). Make sure the Kubernetes `DATABASE_URL` has a user with permission to create databases and apply schema changes. The backend logs will show migration progress; the logs typically record when migrations start and complete.

If your DB is managed (or you prefer migrations to be applied by a separate job), you can disable the programmatic migrations in the backend using the environment variable `RUN_MIGRATIONS=false`. This will prevent the backend from altering the DB schema at startup, allowing an administrator to run migrations in a controlled manner.

Because migrations are compiled into the backend binary now, you no longer need to add SQL files to `backend/migrations`. Instead, add/modify migrations programmatically in `backend/src/db/schema.rs` and ensure your Kubernetes or Docker deployments run the new binary.
