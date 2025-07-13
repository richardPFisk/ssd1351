#!/bin/bash
PORT=$(ls /dev/cu.usbmodem* 2>/dev/null | head -1)
espflash flash --monitor --chip esp32c6 --port "$PORT" "$@"