#!/usr/bin/env bash
set -euo pipefail

decode() { printf '%s' "$1" | base64 -d; }
workspace=$(decode "$1")
profile=$(decode "$2")
command=$(decode "$3")
mount_specs=$(decode "$(decode "$4")")
attestation_specs=$(decode "$(decode "$5")")
attestation_sink=$(decode "$6")
launch_id=$(decode "$7")
manifest_sha=$(decode "$8")
program_id=$(decode "$9")
role_id=$(decode "\${10}")
research_base_sha=$(decode "\${11}")
launch_purpose=$(decode "\${12}")

case "$launch_purpose" in SMOKE|RESEARCH) ;; *) echo "invalid launch purpose" >&2; exit 1 ;; esac
mount --make-rprivate /
mount -t tmpfs -o size=4m,nosuid,nodev,noexec tmpfs /shared
mkdir -p /workspace /shared/research-program

mount_rw_dir() {
  source=$1; target=$2
  [ -d "$source" ] || { echo "missing writable sink: $source" >&2; exit 1; }
  mkdir -p "$target"
  mount --bind "$source" "$target"
}

mount_ro() {
  source=$1; target=$2
  [ -e "$source" ] || { echo "missing source: $source" >&2; exit 1; }
  if [ -d "$source" ]; then mkdir -p "$target"; else mkdir -p "$(dirname "$target")"; touch "$target"; fi
  mount --bind "$source" "$target"
  mount -o remount,bind,ro "$target"
}

mount --bind "$workspace" /workspace
mount_rw_dir "$attestation_sink" /shared/runtime_mount_attestation
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

sha_file() { sha256sum "$1" | awk '{print $1}'; }
recursive_hash() {
  root=$1
  records=''
  while IFS= read -r -d '' file; do
    [ "$(basename "$file")" = 'bundle_manifest.json' ] && continue
    relative=\${file#"$root"/}
    records+="$relative\t$(sha_file "$file")\n"
  done < <(find "$root" -type f -print0 | sort -z)
  printf '%b' "$records" | sort | sha256sum | awk '{print $1}'
}
json_escape() { printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'; }

repo_json=''
auth_json=''
assignment_json=''
all_match=true
while IFS=$'\t' read -r kind source_id mounted_path expected_hash expected_transport; do
  [ -n "$kind" ] || continue
  if [ "$kind" = 'REPO' ]; then
    if [ -d "$mounted_path" ]; then observed=$(recursive_hash "$mounted_path"); else observed=$(sha_file "$mounted_path"); fi
    match=false; [ "$observed" = "$expected_hash" ] && match=true
    [ "$match" = true ] || all_match=false
    [ -n "$repo_json" ] && repo_json+=','
    repo_json+="{\"source_id\":\"$(json_escape "$source_id")\",\"mounted_path\":\"$(json_escape "$mounted_path")\",\"observed_hash\":\"$observed\",\"expected_hash\":\"$expected_hash\",\"match\":$match}"
  elif [ "$kind" = 'AUTH' ]; then
    if [ -d "$mounted_path" ]; then
      observed_transport=$(recursive_hash "$mounted_path")
      observed_content=$(sha_file "$mounted_path/authority.json")
    else
      observed_transport=''
      observed_content=$(sha_file "$mounted_path")
    fi
    match=false
    [ "$observed_content" = "$expected_hash" ] && [ "$observed_transport" = "$expected_transport" ] && match=true
    [ "$match" = true ] || all_match=false
    [ -n "$auth_json" ] && auth_json+=','
    auth_json+="{\"source_id\":\"$(json_escape "$source_id")\",\"mounted_path\":\"$(json_escape "$mounted_path")\",\"observed_bundle_content_identity\":\"$observed_content\",\"observed_directory_transport_hash\":\"$observed_transport\",\"expected_bundle_content_identity\":\"$expected_hash\",\"expected_directory_transport_hash\":\"$expected_transport\",\"match\":$match}"
  elif [ "$kind" = 'ASSIGNMENT' ]; then
    observed=$(sha_file "$mounted_path")
    match=false; [ "$observed" = "$expected_hash" ] && match=true
    [ "$match" = true ] || all_match=false
    assignment_json="{\"source_id\":\"$source_id\",\"mounted_path\":\"$(json_escape "$mounted_path")\",\"observed_hash\":\"$observed\",\"expected_hash\":\"$expected_hash\",\"match\":$match}"
  fi
done <<< "$attestation_specs"

observed_manifest_sha=$(sha_file /shared/research_input_manifest.json)
manifest_match=false
[ "$observed_manifest_sha" = "$manifest_sha" ] && manifest_match=true
[ "$manifest_match" = true ] || all_match=false

probe() { if [ -e "$1" ]; then printf false; else printf true; fi; }
peer_workspace=$(probe /shared/peer_workspace)
peer_assignment=true
own_assignment="/shared/research-program/agent_harness/assignments/$program_id/$role_id.json"
while IFS= read -r candidate; do
  [ "$candidate" = "$own_assignment" ] || peer_assignment=false
done < <(find /shared/research-program/agent_harness/assignments -type f 2>/dev/null)
confirmation=$(probe /shared/confirmation)
legacy=$(probe /shared/legacy)
raw_lake=$(probe /shared/data)
provider_mapping=$(probe /shared/provider_mapping)
windows_mounts=true
if find /mnt -mindepth 1 -print -quit | grep -q .; then windows_mounts=false; fi
for value in "$peer_workspace" "$peer_assignment" "$confirmation" "$legacy" "$raw_lake" "$provider_mapping" "$windows_mounts"; do
  [ "$value" = true ] || all_match=false
done

attestation="{\n  \"schema_version\": \"trinity.runtime-mount-attestation.v1\",\n  \"launch_id\": \"$(json_escape "$launch_id")\",\n  \"launch_purpose\": \"$(json_escape "$launch_purpose")\",\n  \"program_id\": \"$(json_escape "$program_id")\",\n  \"role_id\": \"$(json_escape "$role_id")\",\n  \"research_base_sha\": \"$(json_escape "$research_base_sha")\",\n  \"research_input_manifest_sha256\": \"$manifest_sha\",\n  \"observed_repository_sources\": [$repo_json],\n  \"observed_authority_sources\": [$auth_json],\n  \"observed_assignment\": $assignment_json,\n  \"observed_input_manifest_sha256\": \"$observed_manifest_sha\",\n  \"input_manifest_match\": $manifest_match,\n  \"denial_probes\": {\"peer_workspace\": $peer_workspace, \"peer_assignment\": $peer_assignment, \"confirmation\": $confirmation, \"legacy\": $legacy, \"raw_lake\": $raw_lake, \"provider_mapping\": $provider_mapping, \"windows_mounts\": $windows_mounts},\n  \"all_match\": $all_match\n}\n"
printf '%b' "$attestation" > /shared/runtime_mount_attestation/runtime_mount_attestation.json
mount -o remount,bind,ro /shared/runtime_mount_attestation

[ "$all_match" = true ] || { echo 'RUNTIME_ATTESTATION_FAIL' >&2; exit 42; }

cd /workspace
exec setpriv --reuid=nobody --regid=nogroup --init-groups env -i \
  HOME=/tmp USER=nobody LOGNAME=nobody PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  /bin/bash --noprofile --norc -c "$command"
