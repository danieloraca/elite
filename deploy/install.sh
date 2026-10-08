#!/bin/sh
# Build wirespace-server as danutz first, then: sudo sh deploy/install.sh
set -eu

task_source=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
task_unit=/etc/systemd/system/wirespace.service
task_user=danutz
task_project=/home/danutz/Development/elite
task_binary=$task_source/target/release/wirespace-server

if [ "$(id -u)" -ne 0 ]; then
    echo "Run this installer with sudo." >&2
    exit 1
fi
for task_command in systemctl systemd-analyze ss install curl; do
    command -v "$task_command" >/dev/null
done
task_group=$(id -gn "$task_user")
for task_file in index.html styles.css app.js mq_js_bundle.js elite.wasm favicon.svg THIRD_PARTY.md licenses/LICENSE-MIT licenses/LICENSE-APACHE; do
    test -r "$task_source/dist/$task_file"
done
test -r "$task_source/deploy/wirespace.service"
if [ ! -x "$task_binary" ]; then
    echo "Build the server on the Pi first: cargo build --release --locked --manifest-path $task_source/Cargo.toml -p wirespace-server" >&2
    exit 1
fi

task_existing=$(systemctl show wirespace.service -p FragmentPath --value 2>/dev/null || true)
if [ -n "$task_existing" ] && [ "$task_existing" != "$task_unit" ]; then
    echo "Wirespace is loaded from $task_existing instead of $task_unit. Inspect it before deploying." >&2
    exit 1
fi
if [ -L "$task_unit" ]; then
    echo "$task_unit is a symlink or masked unit. Inspect it before deploying." >&2
    exit 1
fi
if [ -n "$(ss -H -ltn 'sport = :3141')" ] && ! systemctl is-active --quiet wirespace.service; then
    echo "Port 3141 is already in use. No services have been changed." >&2
    exit 1
fi

install -d -o "$task_user" -g "$task_group" -m 0755 "$task_project"
install -d -o "$task_user" -g "$task_group" -m 0755 "$task_project/target/release"
if systemctl is-active --quiet wirespace.service; then
    systemctl stop wirespace.service
fi
if [ "$task_source" != "$task_project" ]; then
    install -d -o "$task_user" -g "$task_group" -m 0755 "$task_project/dist"
    cp -R "$task_source/dist/." "$task_project/dist/"
    install -o "$task_user" -g "$task_group" -m 0755 "$task_binary" "$task_project/target/release/wirespace-server.new"
    mv -f "$task_project/target/release/wirespace-server.new" "$task_project/target/release/wirespace-server"
fi
chown -R "$task_user:$task_group" "$task_project/dist"
find "$task_project/dist" -type d -exec chmod 0755 {} +
find "$task_project/dist" -type f -exec chmod 0644 {} +
if [ -f "$task_unit" ] && ! cmp -s "$task_source/deploy/wirespace.service" "$task_unit"; then
    cp -p "$task_unit" "$task_unit.previous"
fi
install -m 0644 "$task_source/deploy/wirespace.service" "$task_unit"
systemd-analyze verify "$task_unit"
systemctl daemon-reload
systemctl reset-failed wirespace.service
systemctl enable --now wirespace.service
systemctl is-enabled wirespace.service
systemctl is-active wirespace.service

task_page=$(curl --fail --silent --show-error --max-time 2 --retry 10 --retry-connrefused --retry-delay 1 http://127.0.0.1:3141/)
case "$task_page" in
    *'<title>Wirespace'*) ;;
    *) echo "The response is not the Wirespace page." >&2; exit 1 ;;
esac
task_mime=$(curl --fail --silent --show-error --max-time 2 --head --output /dev/null --write-out '%{content_type}' http://127.0.0.1:3141/elite.wasm)
if [ "$task_mime" != application/wasm ]; then
    echo "Incorrect WASM content type: $task_mime" >&2
    exit 1
fi
echo "Wirespace is serving on port 3141 and enabled at boot."
