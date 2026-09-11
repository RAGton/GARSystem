#!/usr/bin/env bash
# Script de lançamento dinâmico para Kryonix OS (NixOS)
# Localiza e injeta as bibliotecas gráficas do nix store no LD_LIBRARY_PATH

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CACHE_FILE="${XDG_CACHE_HOME:-$HOME/.cache}/seniorsystem-nix-ld-path"

find_lib_dir() {
    local pattern="$1"
    for f in $pattern; do
        if [ -f "$f" ] && readelf -h "$f" 2>/dev/null | grep -q "ELF64"; then
            dirname "$f"
            return 0
        fi
    done
    return 1
}

resolve_gui_libs() {
    local dirs=()
    for pat in \
        "/nix/store/*-wayland-*/lib/libwayland-client.so.0" \
        "/nix/store/*-libxkbcommon-*/lib/libxkbcommon.so.0" \
        "/nix/store/*-libglvnd-*/lib/libGL.so.1" \
        "/nix/store/*-vulkan-loader-*/lib/libvulkan.so.1" \
        "/nix/store/*-libx11-*/lib/libX11.so.6" \
        "/nix/store/*-libxcursor-*/lib/libXcursor.so.1" \
        "/nix/store/*-libxi-*/lib/libXi.so.6" \
        "/nix/store/*-libxrandr-*/lib/libXrandr.so.2"
    do
        local d
        d=$(find_lib_dir "$pat" 2>/dev/null || true)
        if [ -n "$d" ]; then
            dirs+=("$d")
        fi
    done
    dirs+=("/run/opengl-driver/lib" "/run/current-system/sw/lib")
    local IFS=":"
    echo "${dirs[*]}"
}

# Verifica cache
NIX_GUI_LIBS=""
if [ -f "$CACHE_FILE" ]; then
    CACHED_PATH="$(cat "$CACHE_FILE")"
    VALID=1
    IFS=':' read -ra DIRS <<< "$CACHED_PATH"
    for d in "${DIRS[@]}"; do
        if [ ! -d "$d" ]; then
            VALID=0
            break
        fi
    done
    if [ "$VALID" -eq 1 ]; then
        NIX_GUI_LIBS="$CACHED_PATH"
    fi
fi

if [ -z "$NIX_GUI_LIBS" ]; then
    NIX_GUI_LIBS="$(resolve_gui_libs)"
    mkdir -p "$(dirname "$CACHE_FILE")"
    echo "$NIX_GUI_LIBS" > "$CACHE_FILE"
fi

export LD_LIBRARY_PATH="${NIX_GUI_LIBS}:${LD_LIBRARY_PATH}"
export WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}"

# Evita warning do Mesa quando DRI_PRIME=0
if [ "$DRI_PRIME" = "0" ]; then
    unset DRI_PRIME
fi

if [ "$#" -gt 0 ]; then
    exec "$@"
else
    TARGET_BIN="$SCRIPT_DIR/target/debug/senior-system-gui"
    if [ ! -f "$TARGET_BIN" ]; then
        cargo build --bin senior-system-gui
    fi
    exec "$TARGET_BIN"
fi
