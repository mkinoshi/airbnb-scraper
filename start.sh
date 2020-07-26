#!/bin/sh

geckodriver --host 0.0.0.0 &
GECKO_PID=$!
echo "Geckodriver started : $GECKO_PID"
sleep 5s
echo "Starting Cargo"
RUST_BACKTRACE=full cargo run
