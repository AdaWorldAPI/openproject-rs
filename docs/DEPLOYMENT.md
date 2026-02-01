# Deployment Guide

This guide covers deploying OpenProject RS in various environments.

## Prerequisites

- PostgreSQL 15+ database
- 512MB+ RAM (recommended: 1GB+)
- Network access to database

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@host:5432/openproject` |
| `SECRET_KEY_BASE` | JWT signing secret (64+ chars) | `openssl rand -hex 64` |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `8080` | Server port |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |
| `DATABASE_POOL_SIZE` | `10` | Connection pool size |

### Storage

| Variable | Default | Description |
|----------|---------|-------------|
| `OPENPROJECT_ATTACHMENTS_STORAGE_PATH` | `/var/openproject/assets` | Local storage path |
| `S3_BUCKET` | - | S3 bucket name |
| `S3_REGION` | `us-east-1` | AWS region |
| `S3_ACCESS_KEY_ID` | - | AWS access key |
| `S3_SECRET_ACCESS_KEY` | - | AWS secret key |
| `S3_ENDPOINT` | - | Custom endpoint (MinIO) |
| `S3_PATH_STYLE` | `false` | Use path-style URLs |

### Email

| Variable | Default | Description |
|----------|---------|-------------|
| `SMTP_HOST` | - | SMTP server hostname |
| `SMTP_PORT` | `587` | SMTP port |
| `SMTP_USERNAME` | - | SMTP username |
| `SMTP_PASSWORD` | - | SMTP password |
| `SMTP_FROM` | - | From email address |
| `SMTP_STARTTLS` | `true` | Enable STARTTLS |
| `SMTP_SSL` | `false` | Use SSL/TLS |

---

## Docker Deployment

### Building the Image

```bash
# Clone repository
git clone https://github.com/AdaWorldAPI/openproject-rs.git
cd openproject-rs

# Build image
docker build -t openproject-rs:latest .
```

### Running with Docker

```bash
# Start PostgreSQL (if not using external)
docker run -d \
  --name openproject-db \
  -e POSTGRES_USER=openproject \
  -e POSTGRES_PASSWORD=openproject \
  -e POSTGRES_DB=openproject \
  -p 5432:5432 \
  postgres:17-alpine

# Start OpenProject RS
docker run -d \
  --name openproject-rs \
  --link openproject-db:db \
  -p 8080:8080 \
  -e DATABASE_URL="postgres://openproject:openproject@db:5432/openproject" \
  -e SECRET_KEY_BASE="$(openssl rand -hex 64)" \
  -e RUST_LOG="info" \
  -v openproject-assets:/var/openproject/assets \
  openproject-rs:latest
```

### Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f server

# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v
```

---

## Railway Deployment

### Via Railway CLI

```bash
# Install Railway CLI
npm install -g @railway/cli

# Login
railway login

# Initialize project
cd openproject-rs
railway init

# Add PostgreSQL database
railway add --database postgres

# Set environment variables
railway variables set SECRET_KEY_BASE="$(openssl rand -hex 64)"

# Deploy
railway up
```

### Via Railway Dashboard

1. Go to [railway.app](https://railway.app)
2. Click "New Project"
3. Select "Deploy from GitHub repo"
4. Choose `AdaWorldAPI/openproject-rs`
5. Railway auto-detects Dockerfile
6. Add PostgreSQL database from "Add Service"
7. Set environment variables in "Variables" tab
8. Deploy

### Railway Environment Variables

Railway automatically provides `DATABASE_URL` when you add a PostgreSQL service. You need to add:

```
SECRET_KEY_BASE=<your-64-char-secret>
RUST_LOG=info
```

---

## Kubernetes Deployment

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: openproject-rs-config
data:
  RUST_LOG: "info"
  HOST: "0.0.0.0"
  PORT: "8080"
```

### Secret

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: openproject-rs-secrets
type: Opaque
stringData:
  DATABASE_URL: "postgres://user:pass@postgres:5432/openproject"
  SECRET_KEY_BASE: "your-64-char-secret-key-here"
```

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: openproject-rs
  labels:
    app: openproject-rs
spec:
  replicas: 3
  selector:
    matchLabels:
      app: openproject-rs
  template:
    metadata:
      labels:
        app: openproject-rs
    spec:
      containers:
      - name: openproject-rs
        image: openproject-rs:latest
        ports:
        - containerPort: 8080
        envFrom:
        - configMapRef:
            name: openproject-rs-config
        - secretRef:
            name: openproject-rs-secrets
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: attachments
          mountPath: /var/openproject/assets
      volumes:
      - name: attachments
        persistentVolumeClaim:
          claimName: openproject-attachments
```

### Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: openproject-rs
spec:
  selector:
    app: openproject-rs
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```

### Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: openproject-rs
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - openproject.example.com
    secretName: openproject-tls
  rules:
  - host: openproject.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: openproject-rs
            port:
              number: 80
```

### PersistentVolumeClaim

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: openproject-attachments
spec:
  accessModes:
    - ReadWriteMany
  storageClassName: standard
  resources:
    requests:
      storage: 10Gi
```

### Deploy to Kubernetes

```bash
# Apply configurations
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml
kubectl apply -f k8s/pvc.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml

# Check status
kubectl get pods -l app=openproject-rs
kubectl logs -l app=openproject-rs
```

---

## Systemd Service

For bare-metal or VM deployments:

### /etc/systemd/system/openproject-rs.service

```ini
[Unit]
Description=OpenProject RS Server
After=network.target postgresql.service

[Service]
Type=simple
User=openproject
Group=openproject
WorkingDirectory=/opt/openproject-rs
ExecStart=/opt/openproject-rs/openproject-server
Restart=always
RestartSec=5

# Environment
Environment=RUST_LOG=info
Environment=HOST=0.0.0.0
Environment=PORT=8080
EnvironmentFile=/etc/openproject-rs/env

# Security
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/openproject

[Install]
WantedBy=multi-user.target
```

### /etc/openproject-rs/env

```bash
DATABASE_URL=postgres://openproject:password@localhost/openproject
SECRET_KEY_BASE=your-64-char-secret-key
OPENPROJECT_ATTACHMENTS_STORAGE_PATH=/var/openproject/assets
```

### Installation

```bash
# Create user
sudo useradd -r -s /bin/false openproject

# Create directories
sudo mkdir -p /opt/openproject-rs /var/openproject/assets /etc/openproject-rs
sudo chown openproject:openproject /var/openproject/assets

# Copy binary
sudo cp target/release/openproject-server /opt/openproject-rs/

# Create env file
sudo nano /etc/openproject-rs/env

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable openproject-rs
sudo systemctl start openproject-rs

# Check status
sudo systemctl status openproject-rs
sudo journalctl -u openproject-rs -f
```

---

## Reverse Proxy Configuration

### Nginx

```nginx
upstream openproject-rs {
    server 127.0.0.1:8080;
    keepalive 32;
}

server {
    listen 443 ssl http2;
    server_name openproject.example.com;

    ssl_certificate /etc/ssl/certs/openproject.crt;
    ssl_certificate_key /etc/ssl/private/openproject.key;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Gzip (disabled - server handles compression)
    gzip off;

    location / {
        proxy_pass http://openproject-rs;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Connection "";

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # WebSocket support (if needed)
    location /cable {
        proxy_pass http://openproject-rs;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}

server {
    listen 80;
    server_name openproject.example.com;
    return 301 https://$server_name$request_uri;
}
```

### Traefik

```yaml
# docker-compose.yml with Traefik
services:
  traefik:
    image: traefik:v2.10
    command:
      - "--providers.docker=true"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock

  server:
    image: openproject-rs:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.openproject.rule=Host(`openproject.example.com`)"
      - "traefik.http.routers.openproject.entrypoints=websecure"
      - "traefik.http.routers.openproject.tls.certresolver=letsencrypt"
```

---

## Monitoring

### Prometheus

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'openproject-rs'
    static_configs:
      - targets: ['openproject-rs:8080']
    metrics_path: /metrics
    scrape_interval: 15s
```

### Grafana Dashboard

Import metrics:
- `http_requests_total`
- `http_requests_by_status`
- `http_request_duration_ms_total`
- `db_queries_total`
- `uptime_seconds`

### Health Checks

```bash
# Liveness (is the process running?)
curl http://localhost:8080/health/live

# Readiness (can it handle traffic?)
curl http://localhost:8080/health/ready

# Full health report
curl http://localhost:8080/health/full
```

---

## Database Migration

When migrating from Ruby OpenProject:

1. **Backup existing database**
   ```bash
   pg_dump openproject > backup.sql
   ```

2. **Deploy OpenProject RS** pointing to same database

3. **Verify API compatibility**
   ```bash
   # Test endpoints
   curl http://new-server:8080/api/v3
   curl http://new-server:8080/api/v3/work_packages
   ```

4. **Gradually route traffic** using load balancer

5. **Monitor for errors** in logs and metrics

---

## Troubleshooting

### Container won't start

```bash
# Check logs
docker logs openproject-rs

# Common issues:
# - DATABASE_URL not set
# - Database not reachable
# - SECRET_KEY_BASE not set
```

### Database connection errors

```bash
# Test database connectivity
docker exec -it openproject-rs sh -c \
  'pg_isready -h db -U openproject'

# Check DATABASE_URL format
# postgres://user:password@host:port/database
```

### High memory usage

```bash
# Reduce connection pool
DATABASE_POOL_SIZE=5

# Enable debug logging to find issues
RUST_LOG=debug
```

### Slow responses

```bash
# Check database query times
curl http://localhost:8080/health/full | jq '.components[] | select(.name=="database")'

# Check metrics for bottlenecks
curl http://localhost:8080/metrics.json | jq '.database'
```
