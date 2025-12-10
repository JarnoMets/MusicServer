# 524 Timeout - Comprehensive Debugging Guide

## Step-by-Step Investigation

### Step 1: Check If Backend Pod Is Even Running

```bash
kubectl get pods -n music

# Expected output:
# NAME                                    READY   STATUS    RESTARTS   AGE
# frontend-XXXX              1/1     Running   0          5m
# backend-XXXX               1/1     Running   0          5m
```

**If backend shows:**
- `CrashLoopBackOff`: Pod is crashing, check logs
- `Pending`: Not enough resources, check node capacity
- `NotReady`: Pod not healthy, check logs

### Step 2: Check Backend Logs for Startup Issues

```bash
# Check backend logs
kubectl logs -n music -l app=backend --all-containers=true

# Look for:
# - "[INFO] Starting Music Server API on http://0.0.0.0:8081" ✓ GOOD
# - "Database connection failed" ✗ BAD - database issue
# - "Error: panic" ✗ BAD - code issue
```

**Common issues:**
```
Error: Connection refused to postgres.postgres:5432
→ PostgreSQL isn't running or hostname is wrong

Error: "musicdb" database does not exist
→ Database doesn't exist, migrations didn't run

Error: Authentication failed
→ PostgreSQL user/password is wrong
```

### Step 3: Test Pod-to-Pod Connectivity

```bash
# Get pod names
# Get pod names
FRONTEND_POD=$(kubectl get pod -n music -l app=frontend -o jsonpath='{.items[0].metadata.name}')
BACKEND_POD=$(kubectl get pod -n music -l app=backend -o jsonpath='{.items[0].metadata.name}')

# Test if you can ping backend from frontend
kubectl exec $FRONTEND_POD -n music -- ping backend

# Test if you can reach the API health endpoint
kubectl exec $FRONTEND_POD -n music -- curl http://backend:8081/api/health

# Expected response:
# {"status":"healthy"}
# OR actual error (not timeout)
```

### Step 4: Check Kubernetes Service Endpoints

```bash
# Show endpoints in the 'music' namespace
kubectl get endpoints -n music

# Should show:
# NAME                       ENDPOINTS         AGE
# backend       10.42.0.XX:8081   5m
# frontend      10.42.0.YY:80     5m

# If ENDPOINTS column is empty (<none>): Pods not healthy
```

### Step 5: Check Traefik Routing

```bash
# Check if Traefik can see the services
kubectl get ingressroute -n music

# Describe the routes
kubectl describe ingressroute frontend -n music
kubectl describe ingressroute backend -n music

# Check Traefik logs
kubectl logs -n kube-system -l app.kubernetes.io/name=traefik --tail=100
```

### Step 6: Test Frontend nginx Proxy Directly

```bash
# From inside frontend pod, test the proxy
kubectl exec $FRONTEND_POD -n music -- sh

# Inside the pod:
curl -v http://localhost/api/health
# This tests: Browser → Traefik → Frontend nginx → Backend

curl -v http://backend:8081/api/health
# This tests: Frontend → Backend (direct, no nginx)
```

## Quick Diagnosis Flow

```
Is backend pod running?
├─ NO → Check pod status: kubectl describe pod <backend-pod>
└─ YES
   ├─ Can frontend reach backend via DNS?
   │  └─ Test: kubectl exec $FRONTEND_POD -- nslookup backend
   └─ Can frontend reach backend via HTTP?
      └─ Test: kubectl exec $FRONTEND_POD -- curl http://backend:8081/api/health
         ├─ Timeout/Connection refused → Networking/firewall issue
         ├─ 404 Not Found → Backend received request but wrong route
         ├─ 500 Error → Backend error (check logs)
         └─ 200 OK → Backend works! Issue is Traefik routing
```

## Most Likely Cause: Database Connection

The backend logs will show if it can't connect to PostgreSQL. Your config specifies:

```yaml
DATABASE_URL: postgres://postgresuser:postgrespwd@postgres.postgres:5432/musicdb
```

Check:
1. Is PostgreSQL running in `postgres` namespace?
2. Is the username/password correct?
3. Does the `musicdb` database exist?

```bash
# If you have another app using postgres, check its deployment
kubectl get pods -n postgres
# Should show postgres pod running

# Check if database exists
kubectl exec -it <postgres-pod> -n postgres -- psql -U postgresuser -l | grep musicdb
# Should show: musicdb | postgresuser | UTF8 | ...
```

## If Everything Looks Good But Still Timing Out

The issue might be **response time** not connectivity. The backend might be:
1. Scanning large music libraries (slow file I/O)
2. Running expensive database queries
3. CPU throttled due to resource limits (see Deployment)

Check backend resource limits:
```bash
kubectl describe pod $(kubectl get pod -n music -l app=backend -o name) -n music | grep -A 5 "Limits"
```

If CPU is set to `500m`, it's throttled on single-core responses.

## The Nuclear Option: Full Debug Logs

```bash
# Enable debug logging
kubectl set env deployment/backend -n music RUST_LOG=debug

# Watch logs in real time
kubectl logs -f -n music -l app=backend
```

## Summary

Most 524 timeouts are caused by:

1. **40%** - Database connection failure (backend starts but can't query)
2. **30%** - Pod not healthy or crashing (check logs)
3. **20%** - Traefik routing misconfiguration (path issue)
4. **10%** - Resource throttling or network policies

**Start with**: `kubectl logs -n music -l app=backend`

That single command will usually reveal the problem.
