#!/bin/bash
set -e

echo "1. Rebuilding release binary..."
cargo build --release

echo "2. Preparing tar package..."
rm -rf /tmp/telegram_bot_deploy
mkdir -p /tmp/telegram_bot_deploy
cp target/release/telegram_bot /tmp/telegram_bot_deploy/
cp Dockerfile /tmp/telegram_bot_deploy/
cp docker-compose.yml /tmp/telegram_bot_deploy/
cp .env /tmp/telegram_bot_deploy/
cp .env.example /tmp/telegram_bot_deploy/
cp DEPLOY.md /tmp/telegram_bot_deploy/
cp -r assets /tmp/telegram_bot_deploy/
cp -r src Cargo.toml Cargo.lock /tmp/telegram_bot_deploy/
tar -czf /tmp/telegram_bot_deploy.tar.gz -C /tmp/telegram_bot_deploy .

HOST="192.124.181.128"
USER="root"
PASS='p_oXUl7zXCpy1Z$ZX(1A'
REMOTE_DIR="/opt/telegram_bot"
SSH_OPTS="-F /dev/null -o StrictHostKeyChecking=no -o ConnectTimeout=10 -o BindAddress=100.123.182.188"
SCP_OPTS="-F /dev/null -o StrictHostKeyChecking=no -o ConnectTimeout=10 -o BindAddress=100.123.182.188"

echo "3. Uploading archive to $HOST..."
sshpass -p "$PASS" scp $SCP_OPTS /tmp/telegram_bot_deploy.tar.gz "$USER@$HOST:/root/telegram_bot_deploy.tar.gz"

echo "4. Unpacking and building docker on remote server..."
sshpass -p "$PASS" ssh $SSH_OPTS "$USER@$HOST" "mkdir -p $REMOTE_DIR/data && tar -xzf /root/telegram_bot_deploy.tar.gz -C $REMOTE_DIR && rm /root/telegram_bot_deploy.tar.gz && chmod +x $REMOTE_DIR/telegram_bot && cd $REMOTE_DIR && docker compose build"

echo "5. Restarting containers if running..."
sshpass -p "$PASS" ssh $SSH_OPTS "$USER@$HOST" "docker rm -f tarot_ai_bot 2>/dev/null; cd $REMOTE_DIR && docker compose up -d"

echo "6. Checking remote status..."
sshpass -p "$PASS" ssh $SSH_OPTS "$USER@$HOST" "cd $REMOTE_DIR && docker compose ps && docker logs --tail 30 tarot_ai_bot"

echo "Done!"
