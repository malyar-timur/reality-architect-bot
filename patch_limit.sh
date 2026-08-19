#!/bin/bash
# 1. Изменяем .env и .env.example
sed -i 's/DAILY_FREE_READINGS=3/DAILY_FREE_READINGS=1/g' .env
sed -i 's/DAILY_FREE_READINGS=3/DAILY_FREE_READINGS=1/g' .env.example

# 2. Изменяем дефолтное значение в коде (в handlers.rs, если где-то пишется "3")
# Если мы писали "+1 расклад", это нормально.

# 3. Обновляем базу данных (чтобы все получили 1)
sqlite3 bot.db "UPDATE users SET energy_balance = 1 WHERE energy_balance > 1;"
