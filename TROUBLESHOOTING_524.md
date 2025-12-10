# Troubleshooting 524 Timeout

## Key Findings

1. **Your backend is configured with `/api` scope** - all routes are `/api/*`
2. **Your Traefik ingress routes both:**
   - `/` → frontend service
   - `/api/*` → backend service (port 8081)
3. **Your nginx also tries to proxy `/api` to backend**

## The Actual Problem

When Traefik routes `/api/music` → backend service, the backend receives `/api/music`.
But if there's a **path strip middleware missing**, the request might be malformed.

## Quick Test

Run this from your machine (outside cluster):

```bash
# Test if backend API is reachable directly
curl -v https://music.jarnomets.com/api/health

# You should see either:
# 1. Success: 200 OK with JSON response
# 2. Error: 524 Gateway Timeout (current problem)
# 3. Error: 404 Not Found (path issue)
# 4. Error: 502 Bad Gateway (connectivity issue)
```

## What Each Error Means

- **524**: Request times out - backend isn't responding in time
- **502**: Gateway can't reach backend - routing/networking issue
- **404**: Backend received request but no handler found - PATH ISSUE
- **500**: Backend error - code issue

## Possible Causes for 524

1. **Kubernetes DNS resolution failing** in frontend pod
2. **Backend pod isn't healthy** or listening
3. **Network policy blocking traffic**
4. **Resource limits causing slowness** (CPU/memory throttling)
5. **Backend database connection failing** (hangs on startup)

## Verification Steps

```bash
# 1. Check if backend pod is actually running and healthy
kubectl get pod -n music -l app=backend
# Should show: backend-XXX   1/1   Running

# 2. Check backend logs for startup issues
kubectl logs -n music -l app=backend
# Look for "listening on" or database connection errors

# 3. Check if service has endpoints
kubectl get endpoints -n music backend
# Should show an IP address

# 4. Try to reach backend from frontend pod
kubectl exec $(kubectl get pod -n music -l app=frontend -o name) \
  -n music -- \
  curl http://backend:8081/api/health

# Should either work or give a clear error message
```

## If Backend Logs Show Database Connection Errors

Check the PostgreSQL configuration:
- Is Postgres running in your cluster?
- Is `postgres.postgres:5432` the correct endpoint?
- Check secrets: `kubectl get secret -n music`
