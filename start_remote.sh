#!/bin/bash
set -e
HOST="192.124.181.128"
USER="root"
PASS='p_oXUl7zXCpy1Z$ZX(1A'

sshpass -p "$PASS" ssh -o StrictHostKeyChecking=no "$USER@$HOST" "cd /opt/telegram_bot && docker compose up -d"
sleep 2
sshpass -p "$PASS" ssh -o StrictHostKeyChecking=no "$USER@$HOST" "cd /opt/telegram_bot && docker compose ps && docker compose logs --tail 30"

