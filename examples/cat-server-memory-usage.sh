#!/bin/bash

while true; do
    pid=$(pgrep -f 'examples/target/debug/server')
    if [ -z "$pid" ]; then
        pid=$(pgrep -f 'examples/target/release/server')
    fi

    if [ -n "$pid" ]; then
        ps -o pid,comm,rss --no-headers -p "$pid" | awk '{printf "pid=%s mem=%.2fMB cmd=%s\n",$1,$3/1024,$2}'
    else
        echo 'Server is not running'
    fi

    sleep 1
done
