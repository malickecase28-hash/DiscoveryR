#!/usr/bin/env bash
set -eu

decode() {
    printf '%s' "$1" | base64 -d
}
repo=$(decode "$1")
lake=$(decode "$2")
workspace=$(decode "$3")
command=$(decode "$4")

mkdir -p /shared/research-program /shared/lake /workspace
mount --make-rprivate /
mount --bind "$repo" /shared/research-program
mount --bind "$lake" /shared/lake
mount --bind "$workspace" /workspace
mount -o remount,bind,ro /shared/research-program
mount -o remount,bind,ro /shared/lake
umount -l /mnt/f 2>/dev/null || true
umount -l /mnt/c 2>/dev/null || true
cd /workspace
exec setpriv --reuid=nobody --regid=nogroup --init-groups env HOME=/tmp USER=nobody /bin/bash -lc "$command"
