# Docker Multi-Architecture Builds

## Overview

XNet Docker images support multiple architectures:
- **amd64**: Intel/AMD processors (servers, desktops)
- **arm64**: Apple M1/M2, modern ARMs
- **arm/v7**: Older ARM devices (Raspberry Pi, IoT)

All architectures use a single manifest tag.

## Building Images

### Automatic (GitHub Actions)

Every tag push automatically builds and publishes:
```bash
git tag v1.0.0
git push origin v1.0.0
```

Images are published to Docker Hub:
- `docker.io/xnetcoin/xnet-node:v1.0.0`
- `docker.io/xnetcoin/xnet-node:latest`
- `docker.io/xnetcoin/xnet-node:1.0` (major.minor)

### Manual Build

```bash
# Install buildx if needed
docker buildx create --use

# Build for current platform
docker build -t xnetcoin/xnet-node:dev .

# Build for specific platform
docker buildx build --platform linux/amd64,linux/arm64 \
  -t xnetcoin/xnet-node:dev --load .

# Build and push (requires registry login)
docker buildx build --platform linux/amd64,linux/arm64,linux/arm/v7 \
  -t xnetcoin/xnet-node:dev --push .
```

## Using Images

### Run node with Docker

```bash
# macOS (native arm64 support)
docker run -d \
  --name xnet-node \
  -p 9944:9944 \
  -p 30333:30333 \
  -v xnet-data:/data \
  xnetcoin/xnet-node:latest \
  --base-path=/data --validator

# View logs
docker logs xnet-node

# Stop node
docker stop xnet-node
```

### Docker Compose

```bash
docker-compose -f docker/docker-compose.yml up -d
```

## Security

### Image Scanning

Every release is scanned for vulnerabilities with Trivy:
```bash
trivy image xnetcoin/xnet-node:latest
```

### Image Signing

Images are signed with Cosign. Verify:
```bash
cosign verify --key cosign.pub docker.io/xnetcoin/xnet-node:v1.0.0
```

## Deployment

### Kubernetes

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: xnet-validator
spec:
  containers:
  - name: xnet
    image: docker.io/xnetcoin/xnet-node:latest
    ports:
    - containerPort: 9944
    - containerPort: 30333
    volumeMounts:
    - name: data
      mountPath: /data
    command:
      - xnet-node
      - --base-path=/data
      - --validator
  volumes:
  - name: data
    emptyDir: {}
```

### Docker Swarm

```bash
docker service create \
  --name xnet-validator \
  --publish 9944:9944/tcp \
  --mount type=volume,source=xnet-data,target=/data \
  xnetcoin/xnet-node:latest \
  --base-path=/data --validator
```

## Version Tags

- `latest` - Latest stable release
- `v1.0.0` - Specific version
- `1.0` - Major.minor (latest patch)
- `develop` - Development branch

## Health Checks

All images include health checks:
```bash
docker inspect xnet-node --format='{{json .State.Health}}'
```

Health check verifies RPC endpoint is responsive.

## Storage

Data is stored in `/data` volume. For backup:

```bash
docker run --rm \
  -v xnet-data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/xnet-backup.tar.gz -C /data .
```

Restore:
```bash
docker volume create xnet-data
docker run --rm \
  -v xnet-data:/data \
  -v $(pwd):/backup \
  alpine tar xzf /backup/xnet-backup.tar.gz -C /data
```

## Support

Report image issues at: https://github.com/xnetcoin/xnet/issues
