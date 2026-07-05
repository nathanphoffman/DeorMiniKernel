#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
qemu-system-i386 -kernel kernel.elf
