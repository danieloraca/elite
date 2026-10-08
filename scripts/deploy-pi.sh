#!/bin/sh
set -eu

task_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
task_host=${PI_HOST:-danutz@192.168.0.25}
for task_command in ssh scp mktemp; do
    command -v "$task_command" >/dev/null
done
sh "$task_root/scripts/package-pi.sh"

task_local=$(mktemp -d /tmp/wirespace-deploy.XXXXXXXX)
task_socket=$task_local/ssh
task_connected=no
task_remote=
task_cleanup() {
    task_status=$?
    trap - EXIT
    if [ "$task_connected" = yes ]; then
        if [ -n "$task_remote" ]; then
            ssh -o BatchMode=yes -o ConnectTimeout=5 -S "$task_socket" -- "$task_host" "rm -rf -- '$task_remote'" </dev/null >/dev/null 2>&1 || true
        fi
        ssh -S "$task_socket" -O exit -- "$task_host" </dev/null >/dev/null 2>&1 || true
    fi
    rm -rf -- "$task_local"
    exit "$task_status"
}
trap task_cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM

printf 'Connecting to %s...\n' "$task_host"
ssh -M -S "$task_socket" -o ControlPersist=60 -o ServerAliveInterval=15 -o ServerAliveCountMax=3 -fN -- "$task_host"
task_connected=yes
task_candidate=$(ssh -S "$task_socket" -- "$task_host" 'mktemp -d /tmp/wirespace-deploy.XXXXXXXX')
case "$task_candidate" in
    /tmp/wirespace-deploy.*) ;;
    *) echo "The Pi did not return a valid deployment directory." >&2; exit 1 ;;
esac
task_suffix=${task_candidate#/tmp/wirespace-deploy.}
case "$task_suffix" in
    ''|*[!a-zA-Z0-9]*) echo "The Pi returned an invalid deployment directory." >&2; exit 1 ;;
esac
task_remote=$task_candidate

printf 'Uploading the browser build and server source...\n'
scp -o "ControlPath=$task_socket" -- "$task_root/target/wirespace-pi.tar.gz" "$task_host:$task_remote/package.tar.gz"
ssh -t -S "$task_socket" -- "$task_host" "set -eu; tar -xzf '$task_remote/package.tar.gz' -C '$task_remote'; sh '$task_remote/deploy/apply.sh'"
