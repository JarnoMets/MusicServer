#!/bin/zsh
set -euo pipefail

# Local dev runner for backend (Actix) and frontend (Vite)
# - Starts backend, waits for health endpoint
# - Starts frontend with VITE_API_URL pointing to local backend
# - Logs are written to ./logs/{backend,frontend}.log
# - Ctrl-C stops both processes

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
LOG_DIR="$REPO_ROOT/logs"
mkdir -p "$LOG_DIR"

cd "$REPO_ROOT"

# Start backend
echo "Starting backend... (logs: $LOG_DIR/backend.log)"
cd backend
cargo build --release >/dev/null || true
cargo run >"$LOG_DIR/backend.log" 2>&1 &
BACKEND_PID=$!
cd "$REPO_ROOT"

# Wait for backend health endpoint
echo "Waiting for backend to become healthy at http://localhost:8081/api/health"
MAX_WAIT=30
i=0
until curl -sSf http://localhost:8081/api/health >/dev/null 2>&1; do
  i=$((i+1))
  if [ $i -ge $MAX_WAIT ]; then
    echo "Backend did not become healthy after $MAX_WAIT seconds. See $LOG_DIR/backend.log"
    kill $BACKEND_PID || true
    exit 1
  fi
  sleep 1
done

echo "Backend healthy. Starting frontend... (logs: $LOG_DIR/frontend.log)"
cd frontend
if [ ! -d node_modules ]; then
  echo "Installing frontend dependencies..."
  npm install
fi
# Start Vite with local backend URL
VITE_API_URL=http://localhost:8081/api npm run dev >"$LOG_DIR/frontend.log" 2>&1 &
FRONTEND_PID=$!
cd "$REPO_ROOT"

# Trap to clean up
cleanup() {
  echo "Shutting down dev servers..."
  if ps -p $FRONTEND_PID >/dev/null 2>&1; then
    kill $FRONTEND_PID
  fi
  if ps -p $BACKEND_PID >/dev/null 2>&1; then
    kill $BACKEND_PID
  fi
  exit 0
}
trap cleanup INT TERM EXIT

echo "Frontend running (Vite). Open: http://localhost:5173"
echo "Backend running at http://localhost:8081"
echo "Logs: $LOG_DIR"

# Wait until processes exit
wait $FRONTEND_PID
wait $BACKEND_PID
