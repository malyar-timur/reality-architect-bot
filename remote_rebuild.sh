#!/bin/bash
set -e
HOST="192.124.181.128"
USER="root"
PASS="wo2oGKd2nNlnvkKqgzC6"
REMOTE_DIR="/opt/telegram_bot"

sshpass -p "$PASS" ssh -o StrictHostKeyChecking=no "$USER@$HOST" "cd $REMOTE_DIR && docker compose build --no-cache && docker compose up -d && sleep 2 && docker compose ps && docker logs --tail 30 tarot_ai_bot"

