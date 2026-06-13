#!/usr/bin/env bash

# Use env vars with defaults
HOST=${LMHA_MQTT_HOST:-"localhost"}
USER=${LMHA_MQTT_USER:-"admin"}
PORT=${LMHA_MQTT_PORT:-"1883"}
PASS=${LMHA_MQTT_PASSWORD:-"changeme"}

mosquitto_sub -v -h "$HOST" -u "$USER" -p "$PORT" -P "$PASS" -t "#"
