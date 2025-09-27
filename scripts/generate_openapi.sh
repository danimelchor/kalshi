#!/usr/bin/env bash

wget https://docs.kalshi.com/openapi.yaml -O /tmp/spec.yaml
cargo progenitor -i /tmp/spec.yaml -o kalshi_api -n kalshi_api -v 0.1.0
rm -rf /tmp/spec.yaml
