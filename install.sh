#!/bin/sh
set -eu

REPO="HamedMP/gurl"
BINARY="gurl"

main() {
    need_cmd uname
    need_cmd mktemp
    need_cmd tar

    version="${GURL_VERSION:-latest}"
    os="$(detect_os)"
    arch="$(detect_arch)"
    target="${arch}-${os}"

    if [ "$version" = "latest" ]; then
        need_cmd curl
        version="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name"' | head -1 | sed 's/.*"v\([^"]*\)".*/\1/')"
    fi

    say "Installing gurl v${version} (${target})"

    url="https://github.com/${REPO}/releases/download/v${version}/${BINARY}-${version}-${target}.tar.gz"
    checksum_url="${url}.sha256"

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    say "Downloading ${url}"
    download "$url" "${tmpdir}/gurl.tar.gz"
    download "$checksum_url" "${tmpdir}/gurl.tar.gz.sha256"

    say "Verifying checksum"
    verify_checksum "${tmpdir}/gurl.tar.gz" "${tmpdir}/gurl.tar.gz.sha256"

    tar xzf "${tmpdir}/gurl.tar.gz" -C "$tmpdir"

    install_dir="$(detect_install_dir)"
    say "Installing to ${install_dir}/gurl"

    if [ -w "$install_dir" ]; then
        mv "${tmpdir}/gurl" "${install_dir}/gurl"
    else
        sudo mv "${tmpdir}/gurl" "${install_dir}/gurl"
    fi

    chmod +x "${install_dir}/gurl"

    say "gurl v${version} installed successfully"

    if ! command -v gurl > /dev/null 2>&1; then
        warn "${install_dir} is not in your PATH"
        warn "Add it: export PATH=\"${install_dir}:\$PATH\""
    fi
}

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "unknown-linux-gnu" ;;
        Darwin*) echo "apple-darwin" ;;
        *)       err "Unsupported OS: $(uname -s)" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)             err "Unsupported architecture: $(uname -m)" ;;
    esac
}

detect_install_dir() {
    if [ -d "/usr/local/bin" ] && [ -w "/usr/local/bin" ]; then
        echo "/usr/local/bin"
    elif [ -d "/usr/local/bin" ]; then
        echo "/usr/local/bin"
    else
        dir="${HOME}/.local/bin"
        mkdir -p "$dir"
        echo "$dir"
    fi
}

download() {
    url="$1"
    dest="$2"
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL "$url" -o "$dest"
    elif command -v wget > /dev/null 2>&1; then
        wget -qO "$dest" "$url"
    else
        err "Need curl or wget to download files"
    fi
}

verify_checksum() {
    file="$1"
    checksum_file="$2"
    expected="$(awk '{print $1}' "$checksum_file")"

    if command -v sha256sum > /dev/null 2>&1; then
        actual="$(sha256sum "$file" | awk '{print $1}')"
    elif command -v shasum > /dev/null 2>&1; then
        actual="$(shasum -a 256 "$file" | awk '{print $1}')"
    else
        warn "No sha256 tool found, skipping checksum verification"
        return 0
    fi

    if [ "$actual" != "$expected" ]; then
        err "Checksum mismatch: expected ${expected}, got ${actual}"
    fi
}

say() {
    printf "  \033[1;32m>\033[0m %s\n" "$1"
}

warn() {
    printf "  \033[1;33m!\033[0m %s\n" "$1" >&2
}

err() {
    printf "  \033[1;31mx\033[0m %s\n" "$1" >&2
    exit 1
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        err "Required command not found: $1"
    fi
}

main "$@"
