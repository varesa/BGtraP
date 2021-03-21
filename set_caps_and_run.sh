#!/bin/sh
set -euo pipefail

if ! ps aux | grep $(ps -o ppid= $$) | grep -q 'cargo test' ; then
    echo "Adding CAP_NET_ADMIN to '$1'"
    sudo setcap cap_net_bind_service=ep "$1"
else
    echo "Skipping capabilities due to running under test harness"
fi

echo "Executing '$1'"
"$@"
