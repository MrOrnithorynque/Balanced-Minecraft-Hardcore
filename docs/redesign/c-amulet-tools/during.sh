#!/bin/bash
# Called by boot_server.sh once the server logged Done: <port> <mc version> <loader>
exec node "$(dirname "$0")/amulet_scenario.js" "$1" "$2" "$3"
