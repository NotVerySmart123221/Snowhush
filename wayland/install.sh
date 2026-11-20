#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${HOME}/.local"
CONFIG_DIR="${XDG_CONFIG_HOME:-${HOME}/.config}"
DATA_DIR="${XDG_DATA_HOME:-${HOME}/.local/share}"
DO_BUILD=1
DO_UNINSTALL=0
DO_PURGE=0
APPLY_BINDS=0

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS]

Install or uninstall Snowhush for Wayland (daemon, control scripts, Quickshell UI).

Options:
  --prefix <DIR>   Install prefix for binaries/scripts (default: ~/.local)
  --no-build       Skip 'cargo build --release' (use existing release binary)
  --apply-binds    Append keybinds & window rules to detected WM config (idempotent)
  --uninstall      Remove installed binaries, scripts, UI, desktop entry, and applied binds
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
        --apply-binds)
            APPLY_BINDS=1
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

remove_marked_block() {
    local file="$1"
    local start_mark="$2"
    local end_mark="$3"
    [[ -f "$file" ]] || return 0
    if grep -Fq "$start_mark" "$file"; then
        local tmp
        tmp="$(mktemp)"
        awk -v s="$start_mark" -v e="$end_mark" '
            index($0, s) { skip = 1; next }
            index($0, e) { skip = 0; next }
            !skip { print }
        ' "$file" > "$tmp"
        mv "$tmp" "$file"
        echo "Removed Snowhush binds from ${file}"
    fi
}

append_marked_block() {
    local file="$1"
    local start_mark="$2"
    local content="$3"
    if grep -Fq "$start_mark" "$file" || grep -Fq "snowhush-toggle" "$file"; then
        echo "Snowhush binds already present in ${file}, skipping."
        return 0
    fi
    printf '\n%s\n' "$content" >> "$file"
    echo "Added Snowhush binds to ${file}"
}

apply_wm_binds() {
    if [[ -n "${HYPRLAND_INSTANCE_SIGNATURE:-}" ]]; then
        local conf="${CONFIG_DIR}/hypr/hyprland.conf"
        if [[ -f "$conf" ]]; then
            append_marked_block "$conf" "# >>> snowhush" "# >>> snowhush
layerrule = above_lock 1, match:namespace ^(snowhush)$
windowrule = float on, center on, size 780 620, match:class .*snowhush-settings.*
bindl = ALT, S, exec, ${BIN_DIR}/snowhush-toggle
bindl = CTRL ALT, S, exec, ${BIN_DIR}/snowhush-settings
# <<< snowhush"
        else
            echo "Hyprland config ${conf} not found (custom/Lua setup?). Add binds manually — see wayland/README.md."
        fi
    elif [[ -n "${SWAYSOCK:-}" ]]; then
        local conf="${CONFIG_DIR}/sway/config"
        if [[ -f "$conf" ]]; then
            append_marked_block "$conf" "# >>> snowhush" "# >>> snowhush
for_window [app_id=\"snowhush-settings\"] floating enable, resize set 780 620, move position center
bindsym Mod1+s exec ${BIN_DIR}/snowhush-toggle
bindsym Control+Mod1+s exec ${BIN_DIR}/snowhush-settings
# <<< snowhush"
        else
            echo "Sway config ${conf} not found. Add binds manually — see wayland/README.md."
        fi
    elif [[ -n "${NIRI_SOCKET:-}" ]]; then
        local conf="${CONFIG_DIR}/niri/config.kdl"
        if [[ -f "$conf" ]]; then
            if grep -Fq "// >>> snowhush" "$conf" || grep -Fq "snowhush-toggle" "$conf"; then
                echo "Snowhush rules/binds already present in ${conf}, skipping."
            else
                printf '\n// >>> snowhush\nwindow-rule {\n    match app-id="snowhush-settings"\n    open-floating true\n    default-column-width { fixed 780; }\n    default-window-height { fixed 620; }\n}\n// <<< snowhush\n' >> "$conf"
                echo "Added Snowhush window-rule to ${conf}"
                if grep -Eq '^[[:space:]]*binds[[:space:]]*\{' "$conf"; then
                    echo "Add inside your existing 'binds { ... }' block in ${conf}:"
                    echo "  Alt+S { spawn \"${BIN_DIR}/snowhush-toggle\"; }"
                    echo "  Ctrl+Alt+S { spawn \"${BIN_DIR}/snowhush-settings\"; }"
                else
                    printf '\n// >>> snowhush-binds\nbinds {\n    Alt+S { spawn "%s/snowhush-toggle"; }\n    Ctrl+Alt+S { spawn "%s/snowhush-settings"; }\n}\n// <<< snowhush-binds\n' "$BIN_DIR" "$BIN_DIR" >> "$conf"
                    echo "Added Snowhush binds to ${conf}"
                fi
            fi
        else
            echo "Niri config ${conf} not found. Add binds manually — see wayland/README.md."
        fi
    elif [[ -f "${CONFIG_DIR}/river/init" ]]; then
        local conf="${CONFIG_DIR}/river/init"
        append_marked_block "$conf" "# >>> snowhush" "# >>> snowhush
riverctl rule-add -app-id 'snowhush-settings' float
riverctl map normal Alt S spawn ${BIN_DIR}/snowhush-toggle
riverctl map normal Control+Alt S spawn ${BIN_DIR}/snowhush-settings
# <<< snowhush"
    else
        echo "No supported compositor config detected for --apply-binds. See wayland/README.md."
    fi
}

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
    remove_marked_block "${CONFIG_DIR}/hypr/hyprland.conf" "# >>> snowhush" "# <<< snowhush"
    remove_marked_block "${CONFIG_DIR}/sway/config" "# >>> snowhush" "# <<< snowhush"
    remove_marked_block "${CONFIG_DIR}/niri/config.kdl" "// >>> snowhush" "// <<< snowhush"
    remove_marked_block "${CONFIG_DIR}/niri/config.kdl" "// >>> snowhush-binds" "// <<< snowhush-binds"
    remove_marked_block "${CONFIG_DIR}/river/init" "# >>> snowhush" "# <<< snowhush"
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
if [[ "$APPLY_BINDS" -eq 1 ]]; then
    apply_wm_binds
elif [[ -n "${HYPRLAND_INSTANCE_SIGNATURE:-}" ]]; then
    echo "Hyprland detected. Run with --apply-binds or add to your config:"
    echo "  layerrule = above_lock 1, match:namespace ^(snowhush)$"
    echo "  windowrule = float on, center on, size 780 620, match:class .*snowhush-settings.*"
    echo "  bindl = ALT, S, exec, snowhush-toggle"
    echo "  bindl = CTRL ALT, S, exec, snowhush-settings"
elif [[ -n "${SWAYSOCK:-}" ]]; then
    echo "Sway detected. Run with --apply-binds or add to your config:"
    echo "  for_window [app_id=\"snowhush-settings\"] floating enable, resize set 780 620, move position center"
    echo "  bindsym Mod1+s exec snowhush-toggle"
    echo "  bindsym Control+Mod1+s exec snowhush-settings"
elif [[ -n "${NIRI_SOCKET:-}" ]]; then
    echo "Niri detected. Run with --apply-binds or see wayland/README.md for KDL rules/binds."
else
    echo "Run 'snowhush-toggle' to toggle snowfall, 'snowhush-settings' for the GUI."
fi
