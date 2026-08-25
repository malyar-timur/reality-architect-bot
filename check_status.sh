#!/bin/bash
set -e
HOST="192.124.181.128"
USER="root"
PASS='p_oXUl7zXCpy1Z$ZX(1A'

sshpass -p "$PASS" ssh -o StrictHostKeyChecking=no "$USER@$HOST" "docker ps && echo '--- LOGS ---' && docker logs --tail 30 tarot_ai_bot"
