#!/usr/bin/env bash
# build-release.sh — cross-compile ask and package into deb, AppImage, .tar.xz
set -euo pipefail

VERSION="0.1.0"
OUTDIR="$(cd "$(dirname "$0")/.." && pwd)/release"
SCRIPTDIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT="$(cd "$(dirname "$0")/.." && pwd)"
ASK_CRATE="${PROJECT}/ask"

APPIMAGETOOL="$(command -v appimagetool 2>/dev/null || echo '')"
if [[ -z "${APPIMAGETOOL}" ]]; then
    echo "[!] appimagetool not found in PATH"
    echo "    Install: cp -a squashfs-root/* ~/.local/share/appimagetool/"
    echo "           ln -sf ~/.local/share/appimagetool/AppRun ~/.local/bin/appimagetool"
    exit 1
fi

mkdir -p "${OUTDIR}"

log() { echo "[*] $*"; }
err() { echo "[!] $*" >&2; }

build_target() {
    local target="$1"
    local arch="$2"
    local binary="${ASK_CRATE}/target/${target}/release/ask"

    if [[ ! -x "$binary" ]]; then
        log "Building ${target} ..."
        (cd "${ASK_CRATE}" && cargo zigbuild --release --target "${target}")
    else
        log "Binary for ${target} already exists, skipping build"
    fi
}

pack_deb() {
    local arch="$1"
    local target="$2"
    local binary="${ASK_CRATE}/target/${target}/release/ask"
    local deb_arch
    case "$arch" in
        amd64)  deb_arch="amd64" ;;
        arm64)  deb_arch="arm64" ;;
        *)      err "Unknown DEB arch: $arch"; return 1 ;;
    esac

    local pkgname="ask_${VERSION}_${deb_arch}"
    local debdir="/tmp/ask-deb-${arch}"
    rm -rf "$debdir"
    mkdir -p "${debdir}/DEBIAN"
    mkdir -p "${debdir}/usr/bin"
    mkdir -p "${debdir}/usr/share/doc/ask"

    cat > "${debdir}/DEBIAN/control" <<EOF
Package: ask
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${deb_arch}
Maintainer: ywnh1
Description: Chat with AI from your terminal
 A minimal CLI tool that lets you chat with AI models
 (DeepSeek, OpenAI-compatible APIs) directly from your shell.
 Supports streaming responses, conversation history, and file context.
EOF

    cp "$binary" "${debdir}/usr/bin/ask"
    chmod 755 "${debdir}/usr/bin/ask"
    cp "${PROJECT}/LICENSE"   "${debdir}/usr/share/doc/ask/"
    cp "${PROJECT}/README.md" "${debdir}/usr/share/doc/ask/"

    dpkg-deb --build --root-owner-group "${debdir}" "${OUTDIR}/${pkgname}.deb" 2>&1
    log "deb: ${OUTDIR}/${pkgname}.deb"
    rm -rf "$debdir"
}

pack_tarxz() {
    local arch="$1"
    local target="$2"
    local binary="${ASK_CRATE}/target/${target}/release/ask"
    local tarname="ask-v${VERSION}-${arch}-linux.tar.xz"
    local tardir="/tmp/ask-tar-${arch}"
    rm -rf "$tardir"
    mkdir -p "${tardir}/ask-v${VERSION}-${arch}-linux"

    cp "$binary"                      "${tardir}/ask-v${VERSION}-${arch}-linux/ask"
    cp "${PROJECT}/LICENSE"           "${tardir}/ask-v${VERSION}-${arch}-linux/"
    cp "${PROJECT}/README.md"         "${tardir}/ask-v${VERSION}-${arch}-linux/"
    cp "${PROJECT}/README_zh.md"      "${tardir}/ask-v${VERSION}-${arch}-linux/"

    chmod 755 "${tardir}/ask-v${VERSION}-${arch}-linux/ask"

    (cd "/tmp/ask-tar-${arch}" && tar -cJf "${OUTDIR}/${tarname}" "ask-v${VERSION}-${arch}-linux")
    log "tar.xz: ${OUTDIR}/${tarname}"
    rm -rf "$tardir"
}

pack_appimage() {
    local arch="$1"
    local target="$2"
    local binary="${ASK_CRATE}/target/${target}/release/ask"
    local appdir="/tmp/ask-appimage-${arch}"
    rm -rf "$appdir"
    mkdir -p "${appdir}/usr/bin"
    mkdir -p "${appdir}/usr/share/doc/ask"
    mkdir -p "${appdir}/usr/share/icons/hicolor/256x256/apps"

    cp "$binary"                      "${appdir}/usr/bin/ask"
    chmod 755 "${appdir}/usr/bin/ask"
    cp "${PROJECT}/LICENSE"           "${appdir}/usr/share/doc/ask/"
    cp "${PROJECT}/README.md"         "${appdir}/usr/share/doc/ask/"
    cp "${SCRIPTDIR}/AppRun"          "${appdir}/AppRun"
    cp "${SCRIPTDIR}/ask.desktop"     "${appdir}/ask.desktop"
    cp "${SCRIPTDIR}/ask.svg"         "${appdir}/ask.svg"
    cp "${SCRIPTDIR}/ask.svg"         "${appdir}/usr/share/icons/hicolor/256x256/apps/ask.svg"

    chmod 755 "${appdir}/AppRun"

    # Set architecture and run appimagetool (without FUSE)
    local appimg_name="ask-v${VERSION}-${arch}-linux.AppImage"
    ARCH="${arch}" "${APPIMAGETOOL}" --no-appstream "${appdir}" "${OUTDIR}/${appimg_name}" 2>&1
    log "AppImage: ${OUTDIR}/${appimg_name}"
    rm -rf "$appdir"
}

# ─── Main ────────────────────────────────────────────────────────────

TARGETS=(
    "x86_64-unknown-linux-musl:amd64"
    "aarch64-unknown-linux-musl:arm64"
)

for entry in "${TARGETS[@]}"; do
    target="${entry%%:*}"
    arch="${entry##*:}"

    log "=== ${target} (${arch}) ==="
    build_target "$target" "$arch"
    pack_deb     "$arch" "$target"
    pack_tarxz   "$arch" "$target"
    pack_appimage "$arch" "$target"
done

log "All packages in ${OUTDIR}/"
ls -lh "${OUTDIR}/"
