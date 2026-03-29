#!/usr/bin/env bash
touch "/tmp/.sigil_recall_$(echo "$PWD" | md5sum | cut -c1-8)"
