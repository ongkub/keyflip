#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p app/build
cp app/Tests/CoreTests.swift app/build/main.swift
swiftc -module-cache-path app/build/module-cache -swift-version 5 app/Sources/Core.swift app/build/main.swift -o app/build/coretests
app/build/coretests
