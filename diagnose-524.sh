#!/bin/bash
# Diagnostic script to debug the 524 timeout issue

echo "=== KUBERNETES DEPLOYMENT STATUS ==="
kubectl get pods -n music
echo ""

echo "=== SERVICE ENDPOINTS ==="
kubectl get svc -n music
echo ""

echo "=== CHECKING FRONTEND POD LOGS ==="
FRONTEND_POD=$(kubectl get pod -n music -l app=frontend -o jsonpath='{.items[0].metadata.name}')
echo "Frontend pod: $FRONTEND_POD"
kubectl logs $FRONTEND_POD -n music --tail=50
echo ""

echo "=== CHECKING BACKEND POD LOGS ==="
BACKEND_POD=$(kubectl get pod -n music -l app=backend -o jsonpath='{.items[0].metadata.name}')
echo "Backend pod: $BACKEND_POD"
kubectl logs $BACKEND_POD -n music --tail=50
echo ""

echo "=== TESTING DNS RESOLUTION FROM FRONTEND POD ==="
kubectl exec $FRONTEND_POD -n music -- nslookup backend
echo ""

echo "=== TESTING DIRECT CONNECTIVITY FROM FRONTEND TO BACKEND ==="
kubectl exec $FRONTEND_POD -n music -- curl -v http://backend:8081/api/music
echo ""

echo "=== TESTING NGINX PROXY ==="
kubectl exec $FRONTEND_POD -n music -- curl -v http://localhost/api/music
echo ""

echo "=== TRAEFIK LOGS ==="
kubectl logs -n kube-system -l app.kubernetes.io/name=traefik --tail=50
echo ""

echo "Done. Review output above for connection issues."
