.PHONY: help dev build-images deploy clean

help:
	@echo "Available commands:"
	@echo "  make dev           - Run development servers"
	@echo "  make build-images  - Build Docker images for k3s"
	@echo "  make deploy        - Deploy to k3s cluster"
	@echo "  make clean         - Clean up k8s resources"

dev:
	@echo "Starting development servers..."
	@echo "Backend: http://localhost:8081"
	@echo "Frontend: http://localhost:5173"
	docker-compose up

build-images:
	@echo "Building Docker images..."
	docker build -t music-server-backend:latest ./backend
	docker build -t music-server-frontend:latest ./frontend
	@echo "Importing images to k3s..."
	k3s ctr images import music-server-backend:latest || docker save music-server-backend:latest | sudo k3s ctr images import -
	k3s ctr images import music-server-frontend:latest || docker save music-server-frontend:latest | sudo k3s ctr images import -

deploy: build-images
	@echo "Deploying to k3s..."
	kubectl apply -f k8s/namespace.yaml
	kubectl apply -f k8s/backend-deployment.yaml
	kubectl apply -f k8s/frontend-deployment.yaml
	kubectl apply -f k8s/ingress.yaml
	kubectl apply -f k8s/ingress-frontend.yaml
	@echo "Deployment complete!"
	@echo "Check status with: kubectl get pods -n music"
	@echo "Note: Uses shared postgres from card-scorer namespace"

clean:
	@echo "Cleaning up k8s resources..."
	kubectl delete -f k8s/ --ignore-not-found=true
	@echo "Cleanup complete!
