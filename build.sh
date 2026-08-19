#!/bin/bash
set -e
cargo build --release
ls -la target/release/telegram_bot
