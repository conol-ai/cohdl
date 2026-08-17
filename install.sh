#!/bin/sh
# CoHDL installer: fetch the newest released `cohdl` binary from GitHub,
# verify its sha256, and install it. One line:
#
#   curl -fsSL https://raw.githubusercontent.com/conol-ai/cohdl/main/install.sh | sh
#
# Environment:
#   COHDL_VERSION      install this exact version (default: newest vX.Y.Z release)
#   COHDL_INSTALL_DIR  install here (default: ~/.cohdl/bin)
#
# The artifact contract is shared with .github/workflows/release-cohdl.yml
# (which produces the artifacts) and `cohdl self-update` (src/selfupdate.rs,
# which consumes them the same way this script does). Windows: no script —
# download the x86_64-pc-windows-msvc archive from the releases page.
set -eu

REPO="conol-ai/cohdl"

err() {
    printf 'install.sh: %s\n' "$1" >&2
    exit 1
}

# Everything lives in main(), invoked on the script's last line: sh reads a
# piped script incrementally, so without the wrapper a connection dropped
# mid-transfer could execute a truncated (semantically different) prefix.
main() {
    # --- platform -> release target triple -----------------------------------
    # Linux installs the static musl build regardless of libc: one artifact
    # per architecture runs on every distribution.
    os="$(uname -s)"
    arch="$(uname -m)"
    case "$os" in
        Darwin)
            case "$arch" in
                arm64) target="aarch64-apple-darwin" ;;
                x86_64) target="x86_64-apple-darwin" ;;
                *) err "unsupported macOS architecture: $arch" ;;
            esac
            ;;
        Linux)
            case "$arch" in
                x86_64) target="x86_64-unknown-linux-musl" ;;
                aarch64 | arm64) target="aarch64-unknown-linux-musl" ;;
                *) err "unsupported Linux architecture: $arch" ;;
            esac
            ;;
        *)
            err "unsupported platform: $os (Windows: download the archive from https://github.com/$REPO/releases)"
            ;;
    esac

    # --- pick the version ----------------------------------------------------
    # Compiler releases are exact `vX.Y.Z` tags; the same repository also
    # carries the VS Code extension's `vscode-v*` releases, so filter by
    # exact-triple shape (pre-release-shaped tags never qualify) and take
    # the numeric maximum rather than trusting list order or "latest".
    # Paginated: a single-page read could be starved once >100 newer
    # extension releases exist.
    if [ -n "${COHDL_VERSION:-}" ]; then
        version="${COHDL_VERSION#v}"
    else
        versions=""
        page=1
        while [ "$page" -le 20 ]; do
            body="$(curl -fsSL "https://api.github.com/repos/$REPO/releases?per_page=100&page=$page")" ||
                err "cannot query the GitHub releases API (offline, or rate-limited?): https://api.github.com/repos/$REPO/releases"
            case "$body" in
                *'"tag_name"'*) ;;
                *) break ;;
            esac
            versions="$versions
$(printf '%s\n' "$body" |
                grep -o '"tag_name": *"v[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*"' |
                cut -d'"' -f4 | sed 's/^v//')"
            page=$((page + 1))
        done
        # Newest first, and a candidate only qualifies if it actually carries
        # this platform's asset under the current naming contract: the
        # pre-rebuild v0.1.0/v0.2.0 releases share the vX.Y.Z tag shape but
        # not the artifact contract, and a release whose assets are still
        # uploading is not installable yet either.
        version=""
        for candidate in $(printf '%s\n' "$versions" | grep . | sort -t. -k1,1nr -k2,2nr -k3,3nr); do
            if curl -fsIL --proto '=https' --tlsv1.2 -o /dev/null \
                "https://github.com/$REPO/releases/download/v$candidate/cohdl-v$candidate-$target.tar.gz"; then
                version="$candidate"
                break
            fi
        done
        [ -n "$version" ] || err "no compiler release found at https://github.com/$REPO/releases"
    fi

    asset="cohdl-v$version-$target.tar.gz"
    base="https://github.com/$REPO/releases/download/v$version"

    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    # An untrapped signal skips the EXIT trap (dash, notably): route the
    # common ones into an exit so the temp dir never outlives an interrupt.
    trap 'exit 1' INT TERM HUP

    printf 'downloading %s ...\n' "$asset"
    curl -fL --proto '=https' --tlsv1.2 -o "$tmp/$asset" "$base/$asset" ||
        err "download failed: $base/$asset (is v$version released for $target?)"
    curl -fsSL --proto '=https' --tlsv1.2 -o "$tmp/sha256sums.txt" "$base/sha256sums.txt" ||
        err "download failed: $base/sha256sums.txt"

    # --- verify --------------------------------------------------------------
    expected="$(awk -v a="$asset" '$2 == a || $2 == "*"a {print $1}' "$tmp/sha256sums.txt")"
    [ -n "$expected" ] || err "sha256sums.txt has no entry for $asset"
    if command -v sha256sum >/dev/null 2>&1; then
        actual="$(sha256sum "$tmp/$asset" | awk '{print $1}')"
    else
        actual="$(shasum -a 256 "$tmp/$asset" | awk '{print $1}')"
    fi
    [ "$actual" = "$expected" ] ||
        err "checksum mismatch for $asset: expected $expected, got $actual"

    # --- install -------------------------------------------------------------
    dir="${COHDL_INSTALL_DIR:-$HOME/.cohdl/bin}"
    mkdir -p "$dir"
    tar -xzf "$tmp/$asset" -C "$tmp" cohdl
    # install(1): atomic-enough replace that also works when $dir/cohdl is
    # the running binary a previous `cohdl self-update` left behind.
    install -m 755 "$tmp/cohdl" "$dir/cohdl"

    # Prove the installed binary actually runs before reporting success (a
    # bare command substitution inside printf would swallow its failure).
    installed_version="$("$dir/cohdl" --version)" ||
        err "the installed binary failed to run: $dir/cohdl --version"
    printf 'installed %s -> %s\n' "$installed_version" "$dir/cohdl"
    case ":$PATH:" in
        *":$dir:"*) ;;
        *)
            printf '\n%s is not on PATH; add it, e.g.:\n' "$dir"
            # The literal `$PATH` below is the instruction being printed.
            # shellcheck disable=SC2016
            printf '  export PATH="%s:$PATH"\n' "$dir"
            ;;
    esac
}

main "$@"
