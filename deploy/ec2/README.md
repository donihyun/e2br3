# EC2 Deployment Bundle

## Files

- `docker-compose.prod.yml`: Production compose file (app only, uses RDS).
- `.env.prod.example`: Template for required runtime environment variables.
- `deploy.sh`: Pull and rollout script used by CD.
- `init-rds.sh`: One-time SQL bootstrap runner for RDS.

## One-time setup on EC2

1. Install Docker and Docker Compose plugin.
2. Create app directory:
   - `sudo mkdir -p /opt/e2br3/schemas`
3. Copy these files to `/opt/e2br3`:
   - `docker-compose.prod.yml`
   - `.env.prod.example` as `.env.prod`
   - `deploy.sh`
4. Make script executable:
   - `chmod +x /opt/e2br3/deploy.sh`
5. Fill `/opt/e2br3/.env.prod` with real secrets and RDS URL.
6. Put XSD files under `/opt/e2br3/schemas/multicacheschemas/...`.
   - Ensure `/opt/e2br3/.env.prod` has `E2BR3_SCHEMAS_DIR=/opt/e2br3/schemas`
     so the container bind-mount includes `/app/schemas/multicacheschemas/...`.

## One-time RDS bootstrap

Run from this repository (local machine or EC2 clone):

```sh
DATABASE_URL='postgres://<user>:<pwd>@<rds-endpoint>:5432/app_db?sslmode=require' \
./deploy/ec2/init-rds.sh
```

Optional: reset DB/user first (destructive, runs `00-recreate-db.sql`):

```sh
RESET_DB=1 \
ROOT_DATABASE_URL='postgres://<admin-user>:<admin-pwd>@<rds-endpoint>:5432/postgres?sslmode=require' \
DATABASE_URL='postgres://<app-user>:<app-pwd>@<rds-endpoint>:5432/app_db?sslmode=require' \
./deploy/ec2/init-rds.sh
```

If you keep DB URLs in `/opt/e2br3/e2br3/deploy/ec2/.env.prod`, you can run:

```sh
cd /opt/e2br3/e2br3
set -a
. /opt/e2br3/e2br3/deploy/ec2/.env.prod
set +a
RESET_DB=1 DATABASE_URL="$SERVICE_DB_URL" ./deploy/ec2/init-rds.sh
```

`init-rds.sh` will use `SERVICE_DB_ROOT_URL` from the env file as `ROOT_DATABASE_URL`.

To skip dev seed data (`13-e2br3-seed.sql`):

```sh
INCLUDE_SEED=0 DATABASE_URL='postgres://<user>:<pwd>@<rds-endpoint>:5432/app_db?sslmode=require' \
./deploy/ec2/init-rds.sh
```

## Manual deploy

```sh
cd /opt/e2br3
IMAGE_REF=ghcr.io/<owner>/e2br3-web-server:<sha> ./deploy.sh
```

## GitHub Actions secrets (for CD deploy job)

- `DEPLOY_HOST`
- `DEPLOY_USER`
- `DEPLOY_SSH_KEY`
- `DEPLOY_COMMAND`:
  - `cd /opt/e2br3 && APP_DIR=/opt/e2br3 ./deploy.sh`
- `GHCR_USERNAME` (optional if host already authenticated)
- `GHCR_TOKEN` (optional if host already authenticated)

The workflow passes `IMAGE_REF` automatically to the remote command.
