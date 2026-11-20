#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${HOME}/.local"
CONFIG_DIR="${XDG_CONFIG_HOME:-${HOME}/.config}"
DATA_DIR="${XDG_DATA_HOME:-${HOME}/.local/share}"
DO_BUILD=1
DO_UNINSTALL=0
DO_PURGE=0

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Install or uninstall Snowhush for Wayland (daemon, control scripts, Quickshell UI).

Options:
  --prefix <DIR>   Install prefix for binaries/scripts (default: ~/.local)
  --no-build       Skip 'cargo build --release' (use existing release binary)
  --uninstall      Remove installed binaries, scripts, UI, and desktop entry
  --purge          With --uninstall, also remove ~/.config/snowhush
  -h, --help       Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix)
            PREFIX="${2:?--prefix requires a directory}"
            shift 2
            ;;
        --no-build)
            DO_BUILD=0
            shift
            ;;
        --uninstall)
            DO_UNINSTALL=1
            shift
            ;;
        --purge)
            DO_PURGE=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

BIN_DIR="${PREFIX}/bin"
QS_DIR="${CONFIG_DIR}/quickshell/snowhush"
APP_DIR="${DATA_DIR}/applications"
DESKTOP_FILE="${APP_DIR}/snowhush-settings.desktop"

if [[ "$DO_UNINSTALL" -eq 1 ]]; then
    echo "Uninstalling Snowhush for Wayland..."
    if [[ -x "${BIN_DIR}/snowhush-toggle" ]]; then
        "${BIN_DIR}/snowhush-toggle" stop >/dev/null 2>&1 || true
    fi
    rm -f "${BIN_DIR}/snowhush-wayland" \
          "${BIN_DIR}/snowhush-toggle" \
          "${BIN_DIR}/snowhush-settings" \
          "${DESKTOP_FILE}"
    rm -rf "${QS_DIR}"
    if [[ "$DO_PURGE" -eq 1 ]]; then
        rm -rf "${CONFIG_DIR}/snowhush"
        echo "Removed config directory: ${CONFIG_DIR}/snowhush"
    fi
    echo "Done."
    exit 0
fi

if [[ "$DO_BUILD" -eq 1 ]]; then
    if ! command -v cargo >/dev/null 2>&1; then
        echo "Error: 'cargo' not found. Install Rust (rustup or distro package) first." >&2
        exit 1
    fi
    echo "Building snowhush-wayland (release)..."
    cargo build --release --manifest-path "${SCRIPT_DIR}/Cargo.toml"
fi

RELEASE_BIN="${SCRIPT_DIR}/target/release/snowhush-wayland"
if [[ ! -x "$RELEASE_BIN" ]]; then
    echo "Error: release binary not found at ${RELEASE_BIN}. Run without --no-build." >&2
    exit 1
fi

echo "Installing to ${BIN_DIR}..."
install -d "${BIN_DIR}" "${QS_DIR}" "${APP_DIR}"
install -m 755 "${RELEASE_BIN}" "${BIN_DIR}/snowhush-wayland"
install -m 755 "${SCRIPT_DIR}/scripts/snowhush-toggle" "${BIN_DIR}/snowhush-toggle"
install -m 755 "${SCRIPT_DIR}/scripts/snowhush-settings" "${BIN_DIR}/snowhush-settings"
install -m 644 "${SCRIPT_DIR}/quickshell/shell.qml" "${QS_DIR}/shell.qml"

cat > "${DESKTOP_FILE}" <<EOF
[Desktop Entry]
Type=Application
Name=Snowhush Settings
Comment=Configure desktop snowfall physics
Exec=${BIN_DIR}/snowhush-settings
Terminal=false
Categories=Settings;Utility;
EOF

echo ""
echo "Installed:"
echo "  ${BIN_DIR}/snowhush-wayland"
echo "  ${BIN_DIR}/snowhush-toggle"
echo "  ${BIN_DIR}/snowhush-settings"
echo "  ${QS_DIR}/shell.qml"
echo "  ${DESKTOP_FILE}"

if ! command -v qs >/dev/null 2>&1; then
    echo ""
    echo "Note: 'qs' (Quickshell) not found in PATH — settings GUI requires Quickshell."
fi

case ":${PATH}:" in
    *":${BIN_DIR}:"*) ;;
    *)
        echo ""
        echo "Note: ${BIN_DIR} is not in your PATH. Add it to your shell profile."
        ;;
esac

echo ""
if [[ -n "${HYPRLAND_INSTANCE_SIGNATURE:-}" ]]; then
    echo "Hyprland detected. Add to your Hyprland config:"
    echo "  layerrule = above_lock 1, match:namespace ^(snowhush)$"
    echo "  windowrule = float on, center on, size 780 620, match:class .*snowhush-settings.*"
    echo "  bindl = ALT, S, exec, snowhush-toggle"
    echo "  bindl = CTRL ALT, S, exec, snowhush-settings"
elif [[ -n "${SWAYSOCK:-}" ]]; then
    echo "Sway detected. Add to your Sway config:"
    echo "  for_window [app_id=\"snowhush-settings\"] floating enable, resize set 780 620, move position center"
    echo "  bindsym Mod1+s exec snowhush-toggle"
    echo "  bindsym Control+Mod1+s exec snowhush-settings"
elif [[ -n "${NIRI_SOCKET:-}" ]]; then
    echo "Niri detected. See wayland/README.md for KDL window-rule and binds."
else
    echo "Run 'snowhush-toggle' to toggle snowfall, 'snowhush-settings' for the GUI."
fi
