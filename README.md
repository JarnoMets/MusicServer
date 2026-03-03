# MusicServer – Self-Hosted Music Management & Streaming

A self-hosted music library, streaming server, and DJ toolset built for personal use. Upload and organise your tracks, manage playlists, stream directly in the browser, download from YouTube, and maintain a clean genre/artist taxonomy.

## Features

### Library Management
- Upload audio files (MP3, FLAC, AAC, WAV, OGG, M4A) — up to 500 MB per request
- Automatic metadata extraction from file tags (title, artist, album, year, BPM, key)
- Duplicate detection via streaming SHA-256 hash before upload
- File sync from a mounted `/music` directory
- Bulk rename / bulk-add-to-playlist via regex

### Streaming & Playback
- In-browser waveform player powered by WaveSurfer.js
- Audio cutting / trimming endpoint
- HTTP range-request streaming with configurable keep-alive
- Internet radio stream management

### Playlists
- Create, reorder, and export playlists
- Export as ZIP archive or Rekordbox XML
- YouTube playlist tracking (auto-sync downloads)

### YouTube Downloader
- Download single videos or full playlists via `yt-dlp`
- Real-time progress via Server-Sent Events
- Auto-download scheduler with configurable rules

### Genre & Metadata System
- Canonical genre taxonomy with aliases and backfill
- Per-artist genre assignment with auto-lookup scheduler
- Discogs-based metadata enrichment (release year, album, style)
- Metadata suggestion queue with accept/reject workflow
- BPM detection (aubio)
- Key detection (Rekordbox-compatible analysis)

### Authentication
- Google OAuth SSO (mobile + web flows)
- JWT session management
- Admin token for protected admin endpoints (SHA-256 stored, constant-time compare)

### Admin
- Audit log with revert support
- Artist renaming (propagates to all tracks)
- Genre merge, alias, and reprocess operations
- Auto-download configuration and manual trigger
- Bulk metadata operations

## Tech Stack

| Layer | Technology |
|---|---|
| Backend | Rust · Actix-web 4 · SQLx |
| Database | PostgreSQL 15 |
| Frontend | Vue 3 · TypeScript · Vite · WaveSurfer.js · Pinia · vue-i18n |
| Audio analysis | aubio (BPM) · symphonia (decoding) · rustfft |
| Deployment | Docker Compose / Kubernetes (k3s) |

## Quick Start

### Local development

```bash
./dev.sh
```

- Backend: http://localhost:8081
- Frontend: http://localhost:5173

### Docker Compose

```bash
docker-compose up --build
```

### Kubernetes (k3s)

```bash
make deploy
```

> **Note:** The Kubernetes deployment mounts a persistent volume at `/music` for audio files. Make sure the PVC is provisioned before deploying.

## Project Structure

```
MusicServer/
├── backend/                    # Rust Actix-web API
│   ├── src/
│   │   ├── db/                 # Database layer & schema migrations
│   │   ├── models/             # Data models
│   │   ├── services/           # Business logic
│   │   │   ├── bpm_service.rs
│   │   │   ├── discogs_service.rs
│   │   │   ├── genre_detection.rs
│   │   │   ├── auto_download_service.rs
│   │   │   ├── auto_genre_lookup_service.rs
│   │   │   ├── auto_metadata_lookup_service.rs
│   │   │   ├── playlist_export_service.rs
│   │   │   ├── rekordbox_service.rs
│   │   │   ├── yt_download_service.rs
│   │   │   └── …
│   │   ├── routes.rs           # All HTTP route handlers
│   │   ├── auth_routes.rs      # Google OAuth + JWT routes
│   │   ├── audit_routes.rs     # Audit log routes
│   │   ├── auth_middleware.rs  # JWT auth middleware
│   │   ├── admin_middleware.rs # Admin token middleware
│   │   ├── yt_downloader.rs    # yt-dlp wrapper
│   │   └── main.rs
│   └── Cargo.toml
├── frontend/                   # Vue 3 SPA
│   ├── src/
│   │   ├── views/              # Browse, Upload, EditTrack, Login
│   │   ├── features/           # Feature modules (music, playlists, genres, artists, decks, …)
│   │   ├── components/         # Shared UI components
│   │   ├── stores/             # Pinia stores
│   │   ├── api/                # Typed API client
│   │   ├── locales/            # i18n (EN / NL)
│   │   └── router/
│   └── package.json
├── k8s/                        # Kubernetes manifests
├── docker-compose.yaml
├── Makefile
├── dev.sh
└── deploy.sh
```

## API Overview

### Authentication
| Method | Path | Description |
|---|---|---|
| GET | `/api/auth/google/url` | Get Google OAuth URL |
| POST | `/api/auth/google/callback` | Exchange code for JWT |
| POST | `/api/auth/google/mobile` | Mobile OAuth flow |
| GET | `/api/me` | Current user |

### Music
| Method | Path | Description |
|---|---|---|
| GET | `/api/music` | List tracks (paginated, filterable) |
| GET | `/api/music/all-cached` | All tracks from cache |
| POST | `/api/music/upload` | Upload audio files |
| GET | `/api/music/:id/stream` | Stream audio (range requests) |
| POST | `/api/music/:id/bpm-detect` | Run BPM detection |
| POST | `/api/music/sync` | Sync from `/music` directory |

### Playlists
| Method | Path | Description |
|---|---|---|
| GET/POST | `/api/playlists` | List / create playlists |
| GET | `/api/playlists/:id/export/zip` | Export as ZIP |
| GET | `/api/playlists/:id/export/rekordbox` | Export as Rekordbox XML |

### YouTube
| Method | Path | Description |
|---|---|---|
| POST | `/api/youtube/download` | Start a download |
| GET | `/api/youtube/stream/:id` | SSE progress stream |

### Admin (requires admin token)
| Method | Path | Description |
|---|---|---|
| POST | `/api/admin/genres` | Create genre |
| POST | `/api/admin/genres/aliases/backfill` | Add alias & backfill |
| PUT | `/api/admin/metadata/config` | Update Discogs config |
| GET | `/api/admin/audit/logs` | View audit log |
| POST | `/api/admin/music/bulk-rename` | Bulk rename by regex |

## Environment Variables

### Backend

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | local postgres | PostgreSQL connection string |
| `JWT_SECRET` | *(insecure default)* | JWT signing secret — **change in production** |
| `GOOGLE_CLIENT_ID` | – | Google OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | – | Google OAuth client secret |
| `APP_URL` | `http://localhost:8081` | Base URL for OAuth callbacks |
| `ADMIN_TOKEN_SHA256` | – | SHA-256 hex of the admin raw token |
| `DISCOGS_TOKEN` | – | Discogs personal access token (optional) |
| `RUST_LOG` | `warn` | Log level |
| `RUN_MIGRATIONS` | `true` | Set to `false` to skip DB migrations at startup |

### Frontend

| Variable | Default | Description |
|---|---|---|
| `VITE_API_URL` | `http://localhost:8081/api` | Backend API base URL |

## Admin Token

The admin token protects genre management, backfill, bulk-operation, and audit endpoints. The server stores only the SHA-256 of the raw token; the raw token is never persisted.

**Generate a token:**
```bash
RAW_TOKEN=$(head -c 32 /dev/urandom | base64)
echo "Raw token: $RAW_TOKEN"
echo -n "$RAW_TOKEN" | sha256sum | awk '{print $1}'
```

**Set the Kubernetes secret:**
```bash
kubectl create secret generic music-admin \
  --namespace music \
  --from-literal=ADMIN_TOKEN_SHA256='<sha256-hex>' \
  --dry-run=client -o yaml | kubectl apply -f -
kubectl rollout restart deployment/backend -n music
```

**Use in API calls:**
```bash
curl -H "Authorization: Bearer $RAW_TOKEN" https://music.example.com/api/admin/genres
```

## Database Migrations

Migrations are compiled into the binary and run automatically at startup via `backend/src/db/schema.rs`. Set `RUN_MIGRATIONS=false` to skip if you manage migrations separately.

## License

MIT
