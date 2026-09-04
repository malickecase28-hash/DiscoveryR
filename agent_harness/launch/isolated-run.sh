#!/usr/bin/env bash
set -eu
decode() { printf '%s' "$1" | base64 -d; }
workspace=$(decode "$1")
profile=$(decode "$2")
command=$(decode "$3")
mount_specs=$(decode "$(decode "$4")")
runtime_endpoint=$(decode "${5:-}")
mkdir -p /workspace /shared/research-program
mount --make-rprivate /
mount --bind "$workspace" /workspace
mount_ro() {
  source=$1; target=$2
  [ -e "$source" ] || { echo "missing source: $source" >&2; exit 1; }
  if [ -d "$source" ]; then mkdir -p "$target"; else mkdir -p "$(dirname "$target")"; touch "$target"; fi
  mount --bind "$source" "$target"
  mount -o remount,bind,ro "$target"
}
while IFS='|' read -r source_b64 target_b64; do
  [ -n "$source_b64" ] || continue
  mount_ro "$(decode "$source_b64")" "$(decode "$target_b64")"
done <<< "$mount_specs"
for mountpoint in /mnt/*; do
  [ -e "$mountpoint" ] || continue
  umount -l "$mountpoint" 2>/dev/null || true
done
mount -t tmpfs -o size=1m,nosuid,nodev,noexec tmpfs /mnt
mount -t tmpfs -o size=1m,nosuid,nodev,noexec tmpfs /home
mount -t tmpfs -o size=1m,nosuid,nodev,noexec tmpfs /root
cd /workspace
exec setpriv --reuid=nobody --regid=nogroup --init-groups env -i \
  HOME=/tmp USER=nobody LOGNAME=nobody PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  TRINITYR_RUNTIME_ENDPOINT="$runtime_endpoint" \
  /bin/bash --noprofile --norc -c "$command"
