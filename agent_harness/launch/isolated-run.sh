#!/usr/bin/env bash
set -eu

decode() {
    printf '%s' "$1" | base64 -d
}
repo=$(decode "$1")
lake=$(if [ "$2" = "-" ]; then printf ''; else decode "$2"; fi)
workspace=$(decode "$3")
command=$(decode "$4")

mkdir -p /shared/research-program /workspace
mount --make-rprivate /
mount --bind "$workspace" /workspace
mount_ro() {
    source=$1
    target=$2
    if [ -e "$source" ]; then
        mkdir -p "$target"
        mount --bind "$source" "$target"
        mount -o remount,bind,ro "$target"
    fi
}
mount_ro "$repo/contracts" /shared/research-program/contracts
mount_ro "$repo/registry" /shared/research-program/registry
mount_ro "$repo/instruments/XAUUSD" /shared/research-program/instruments/XAUUSD
mount_ro "$repo/agent_harness/assignments/AP-001" /shared/research-program/assignments/AP-001
if [ -n "$lake" ]; then
    mkdir -p /shared/lake
    mount --bind "$lake" /shared/lake
    mount -o remount,bind,ro /shared/lake
fi
umount -l /mnt/f 2>/dev/null || true
umount -l /mnt/c 2>/dev/null || true
cd /workspace
exec setpriv --reuid=nobody --regid=nogroup --init-groups env -i \
    HOME=/tmp USER=nobody LOGNAME=nobody \
    PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    /bin/bash --noprofile --norc -c "$command"
