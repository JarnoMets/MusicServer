#!/bin/zsh
set -e

# Compile backend (Rust)
#echo "Compiling backend..."
#(cd backend && cargo build --release)

# Compile frontend (Vite)
#echo "Compiling frontend..."
#(cd frontend && npm install && npm run build)

# Build Docker images
echo "Building backend Docker image..."
sudo docker build -t music-server-backend:latest backend
echo "Building frontend Docker image..."
sudo docker build -t music-server-frontend:latest frontend

echo "Note: Backend contains programmatic Rust migrations that run at startup (backend/src/db/schema.rs)."
echo "Ensure the DATABASE_URL user has permission to create the database and apply schema changes if necessary."
echo "If you need to skip migrations (for example in managed DB environments), set RUN_MIGRATIONS=false in your environment for the backend container."

# Check if logged in to Docker
if ! sudo docker info 2>&1 | grep -q 'Username:'; then
  echo "Not logged in to Docker. Please log in."
  sudo docker login
else
  echo "Already logged in to Docker."
fi

# Tag images
sudo docker tag music-server-backend:latest jarn/music-server-backend:latest
sudo docker tag music-server-frontend:latest jarn/music-server-frontend:latest

# Push images
sudo docker push jarn/music-server-backend:latest
sudo docker push jarn/music-server-frontend:latest

echo "Deployment complete!"
