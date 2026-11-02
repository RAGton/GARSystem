#!/bin/bash
# GAR System GUI launcher
# Resolve o problema do winit/glutin em NixOS com Hyprland+Xwayland rootless
# Carrega as libs do /nix/store que o binario compilado pelo cargo nao acha

set -e

# Libs necessarias (paths podem mudar entre rebuilds do Nix)
NIX_LIB="/nix/store/4wkicwb2jx5h63d79g5qyp9kn4vx27yb-libx11-1.8.13/lib"
NIX_LIB+=":/nix/store/cvr8yyrrqnjxgd7bs6lwgra4qma7kwl9-libxcb-1.17.0/lib"
NIX_LIB+=":/nix/store/ir3ha3ghl8wvbsd4wiv5hdp7vm70zn3y-wayland-1.25.0/lib"
NIX_LIB+=":/nix/store/nvrv5w6rhc39h8snrsw03nqs679cf7rh-libxkbcommon-1.13.2/lib"
NIX_LIB+=":/nix/store/mj9klbkhzal0vc1aazsajrydkimhc0h2-libglvnd-1.7.0/lib"
NIX_LIB+=":/nix/store/71yhcn0hj5mbmjjrvfxp9w2h5s59w71l-mesa-libgbm-26.0.3/lib"
NIX_LIB+=":/nix/store/qccs7f6nfqrjadfw8pag3amq5m8h34zi-libxcursor-1.2.3/lib"
NIX_LIB+=":/nix/store/f6z2258liq5vbiah6jrkvy4iymdq2c5a-libxrender-0.9.12/lib"
NIX_LIB+=":/nix/store/aks94skdqa6mjhwz09w2xfxs9h4j5csa-libxrandr-1.5.5/lib"
NIX_LIB+=":/nix/store/ncvfcirz3d4g2maj5dvzwjwrimwvnhl6-libxi-1.8.3/lib"

export LD_LIBRARY_PATH="$NIX_LIB${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export PATH="/nix/store/gmgfpzc0w2pgby0wmq9r88m39albiyfw-cargo-1.98.1/bin:$PATH"

# Auto-detect: se Xwayland :0 existe, usa X11 (mas glutin falha em rootless!)
# Tenta Wayland primeiro
if [ -S "/run/user/1000/wayland-1" ]; then
    export WAYLAND_DISPLAY=wayland-1
    export GDK_BACKEND=wayland
    export MOZ_ENABLE_WAYLAND=1
fi

# Tenta abrir
cd "$(dirname "$0")"
exec target/debug/gar-system-gui "$@"
