#!/usr/bin/env zsh
#
# seconds_from_now.sh: show timestamp of $1 seconds in the future

delay=$1
epoch=$(date +%s)
date +%H:%M:%S -d @$((${epoch} + ${delay}))
