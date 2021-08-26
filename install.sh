#!/usr/bin/env sh

# Options
#
#   -V, --verbose
#     Enable verbose output for the installer
#
#   -f, -y, --force, --yes
#     Skip the confirmation prompt during installation

TERMSCP_VERSION="0.6.1"
GITHUB_URL="https://github.com/veeso/termscp/releases/download/v${TERMSCP_VERSION}"
DEB_URL="${GITHUB_URL}/termscp_${TERMSCP_VERSION}_amd64.deb"
FREEBSD_URL="${GITHUB_URL}/termscp-${TERMSCP_VERSION}.txz"
RPM_URL="${GITHUB_URL}/termscp-${TERMSCP_VERSION}-1.x86_64.rpm"

set -eu
printf "\n"

BOLD="$(tput bold 2>/dev/null || printf '')"
GREY="$(tput setaf 0 2>/dev/null || printf '')"
UNDERLINE="$(tput smul 2>/dev/null || printf '')"
RED="$(tput setaf 1 2>/dev/null || printf '')"
GREEN="$(tput setaf 2 2>/dev/null || printf '')"
YELLOW="$(tput setaf 3 2>/dev/null || printf '')"
BLUE="$(tput setaf 4 2>/dev/null || printf '')"
MAGENTA="$(tput setaf 5 2>/dev/null || printf '')"
NO_COLOR="$(tput sgr0 2>/dev/null || printf '')"

# Functions

info() {
    printf '%s\n' "${BOLD}${GREY}>${NO_COLOR} $*"
}

warn() {
    printf '%s\n' "${YELLOW}! $*${NO_COLOR}"
}

error() {
    printf '%s\n' "${RED}x $*${NO_COLOR}" >&2
}

completed() {
    printf '%s\n' "${GREEN}✓${NO_COLOR} $*"
}

has() {
    command -v "$1" 1>/dev/null 2>&1
}

get_tmpfile() {
    local suffix
    suffix="$1"
    if has mktemp; then
        printf "%s.%s" "$(mktemp)" "${suffix}"
    else
        # No really good options here--let's pick a default + hope
        printf "/tmp/termscp.%s" "${suffix}"
    fi
}

download() {
    output="$1"
    url="$2"
    
    if has curl; then
        cmd="curl --fail --silent --location --output $output $url"
    elif has wget; then
        cmd="wget --quiet --output-document=$output $url"
    elif has fetch; then
        cmd="fetch --quiet --output=$output $url"
    else
        error "No HTTP download program (curl, wget, fetch) found, exiting…"
        return 1
    fi
    $cmd && return 0 || rc=$?
    
    error "Command failed (exit code $rc): ${BLUE}${cmd}${NO_COLOR}"
    warn "If you believe this is a bug, please report immediately an issue to <https://github.com/veeso/termscp/issues/new>"
    return $rc
}

test_writeable() {
  local path
  path="${1:-}/test.txt"
  if touch "${path}" 2>/dev/null; then
    rm "${path}"
    return 0
  else
    return 1
  fi
}

elevate_priv() {
    if ! has sudo; then
        error 'Could not find the command "sudo", needed to install termscp on your system.'
        info "If you are on Windows, please run your shell as an administrator, then"
        info "rerun this script. Otherwise, please run this script as root, or install"
        info "sudo."
        exit 1
    fi
    if ! sudo -v; then
        error "Superuser not granted, aborting installation"
        exit 1
    fi
}

elevate_priv_ex() {
    check_dir="$1"
    if test_writeable "$check_dir"; then
        sudo=""
    else
        warn "Root permissions are required to install dependecies"
        elevate_priv
        sudo="sudo"
    fi
    echo $sudo
}

# Currently supporting:
#   - macos
#   - linux
#   - freebsd
detect_platform() {
    local platform
    platform="$(uname -s | tr '[:upper:]' '[:lower:]')"
    
    case "${platform}" in
        linux) platform="linux" ;;
        darwin) platform="macos" ;;
        freebsd) platform="freebsd" ;;
    esac
    
    printf '%s' "${platform}"
}

# Currently supporting:
#   - x86_64
detect_arch() {
    local arch
    arch="$(uname -m | tr '[:upper:]' '[:lower:]')"
    
    case "${arch}" in
        amd64) arch="x86_64" ;;
        armv*) arch="arm" ;;
        arm64) arch="aarch64" ;;
    esac
    
    # `uname -m` in some cases mis-reports 32-bit OS as 64-bit, so double check
    if [ "${arch}" = "x86_64" ] && [ "$(getconf LONG_BIT)" -eq 32 ]; then
        arch="i686"
    elif [ "${arch}" = "aarch64" ] && [ "$(getconf LONG_BIT)" -eq 32 ]; then
        arch="arm"
    fi
    
    if [ "${arch}" != "x86_64" ]; then
        error "Unsupported arch ${arch}"
        return 1
    fi
    
    printf '%s' "${arch}"
}

confirm() {
    if [ -z "${FORCE-}" ]; then
        printf "%s " "${MAGENTA}?${NO_COLOR} $* ${BOLD}[y/N]${NO_COLOR}"
        set +e
        read -r yn </dev/tty
        rc=$?
        set -e
        if [ $rc -ne 0 ]; then
            error "Error reading from prompt (please re-run with the '--yes' option)"
            exit 1
        fi
        if [ "$yn" != "y" ] && [ "$yn" != "yes" ]; then
            error 'Aborting (please answer "yes" to continue)'
            exit 1
        fi
    fi
}

# Installers

install_on_bsd() {
    info "Installing termscp via FreeBSD pkg"
    archive=$(get_tmpfile "txz")
    download "${archive}" "${FREEBSD_URL}"
    info "Downloaded FreeBSD package to ${archive}"
    if test_writeable "/usr/local/bin"; then
        sudo=""
        msg="Installing termscp, please wait…"
    else
        warn "Root permissions are required to install termscp…"
        elevate_priv
        sudo="sudo"
        msg="Installing termscp as root, please wait…"
    fi
    info "$msg"
    $sudo pkg install -y "${archive}"
}

install_on_arch_linux() {
    pkg="$1"
    info "Detected ${YELLOW}${pkg}${NO_COLOR} on your system"
    confirm "${YELLOW}rust${NO_COLOR} is required to install ${GREEN}termscp${NO_COLOR}; would you like to proceed?"
    $pkg -S rust
    info "Installing ${GREEN}termscp${NO_COLOR} AUR package…"
    $pkg -S termscp
}

install_on_linux() {
    local msg
    local sudo
    local archive
    if has yay; then
        install_on_arch_linux yay
    elif has pakku; then
        install_on_arch_linux pakku
    elif has paru; then
        install_on_arch_linux paru
    elif has aurutils; then
        install_on_arch_linux aurutils
    elif has pamac; then
        install_on_arch_linux pamac
    elif has pikaur; then
        install_on_arch_linux pikaur
    elif has dpkg; then
        if [ "${ARCH}" != "x86_64" ]; then # It's okay on AUR; not on other distros
            try_with_cargo "we don't distribute packages for ${ARCH} at the moment"
        else
            info "Detected dpkg on your system"
            info "Installing ${GREEN}termscp${NO_COLOR} via Debian package"
            archive=$(get_tmpfile "deb")
            download "${archive}" "${DEB_URL}"
            info "Downloaded debian package to ${archive}"
            if test_writeable "/usr/bin"; then
                sudo=""
                msg="Installing ${GREEN}termscp${NO_COLOR}, please wait…"
            else
                warn "Root permissions are required to install ${GREEN}termscp${NO_COLOR}…"
                elevate_priv
                sudo="sudo"
                msg="Installing ${GREEN}termscp${NO_COLOR} as root, please wait…"
            fi
            info "$msg"
            $sudo dpkg -i "${archive}"
        fi
    elif has rpm; then
        if [ "${ARCH}" != "x86_64" ]; then # It's okay on AUR; not on other distros
            try_with_cargo "we don't distribute packages for ${ARCH} at the moment"
        else
            info "Detected rpm on your system"
            info "Installing ${GREEN}termscp${NO_COLOR} via RPM package"
            archive=$(get_tmpfile "rpm")
            download "${archive}" "${RPM_URL}"
            info "Downloaded rpm package to ${archive}"
            if test_writeable "/usr/bin"; then
                sudo=""
                msg="Installing ${GREEN}termscp${NO_COLOR}, please wait…"
            else
                warn "Root permissions are required to install ${GREEN}termscp${NO_COLOR}…"
                elevate_priv
                sudo="sudo"
                msg="Installing ${GREEN}termscp${NO_COLOR} as root, please wait…"
            fi
            info "$msg"
            $sudo rpm -U "${archive}"
        fi
    else
        try_with_cargo "No suitable installation method found for your Linux distribution; if you're running on Arch linux, please install an AUR package manager (such as yay). Currently only Arch, Debian based and Red Hat based distros are supported"
    fi
}

install_on_macos() {
    if has brew; then
        if has termscp; then
            info "Upgrading ${GREEN}termscp${NO_COLOR}…"
            # The OR is used since someone could have installed via cargo previously
            brew update && brew upgrade termscp || brew install veeso/termscp/termscp
        else
            info "Installing ${GREEN}termscp${NO_COLOR}…"
            brew install veeso/termscp/termscp
        fi
    else
        try_with_cargo "brew is missing on your system; please install it from <https://brew.sh/>"
    fi
}

# -- cargo installation

install_bsd_cargo_deps() {
    set -e
    confirm "${YELLOW}libssh, gcc${NO_COLOR} are required to install ${GREEN}termscp${NO_COLOR}; would you like to proceed?"
    sudo="$(elevate_priv_ex /usr/local/bin)"
    $sudo pkg install -y curl wget libssh gcc
    info "Dependencies installed successfully"
}

install_linux_cargo_deps() {
    local debian_deps="gcc pkg-config libssl-dev libssh2-1-dev libdbus-1-dev"
    local rpm_deps="gcc openssl pkgconfig libdbus-devel openssl-devel"
    local arch_deps="gcc openssl pkg-config dbus"
    local deps_cmd=""
    # Get pkg manager
    if has apt; then
        deps_cmd="apt install -y $debian_deps"
    elif has apt-get; then
        deps_cmd="apt-get install -y $debian_deps"
    elif has yum; then
        deps_cmd="yum -y install $rpm_deps"
    elif has dnf; then
        deps_cmd="dnf -y install $rpm_deps"
    elif has pacman; then
        deps_cmd="pacman -S --noconfirm $arch_deps"
    else
        error "Could not find any suitable package manager for your linux distro 🙄"
        error "Supported package manager are: 'apt', 'apt-get', 'yum', 'dnf', 'pacman'"
        exit 1
    fi
    set -e
    confirm "${YELLOW}libssh, gcc, openssl, pkg-config, libdbus${NO_COLOR} are required to install ${GREEN}termscp${NO_COLOR}. The following command will be used to install the dependencies: '${BOLD}${YELLOW}${deps_cmd}${NO_COLOR}'. Would you like to proceed?"
    sudo="$(elevate_priv_ex /usr/local/bin)"
    $sudo $deps_cmd
    info "Dependencies installed successfully"
}

install_cargo() {
    if has cargo; then
        return 0
    fi
    cargo_env="$HOME/.cargo/env"
    # Check if cargo is already installed (actually), but not loaded
    if [ -f $cargo_env ]; then
        . $cargo_env
    fi
    # Check again cargo
    if has cargo; then
        return 0
    else
        confirm "${YELLOW}rust${NO_COLOR} is required to build termscp with cargo; would you like to install it now?"
        set -e
        rustup=$(get_tmpfile "sh")
        info "Downloading rustup.sh…"
        download "${rustup}" "https://sh.rustup.rs"
        chmod +x $rustup
        $rustup -y
        info "Rust installed with success"
        . $cargo_env
    fi

}

try_with_cargo() {
    err="$1"
    # Install cargo
    install_cargo
    if has cargo; then
        info "Installing ${GREEN}termscp${NO_COLOR} via Cargo…"
        case $PLATFORM in
            "freebsd")
                install_bsd_cargo_deps
                cargo install --no-default-features termscp
            ;;

            "linux")
                install_linux_cargo_deps
                cargo install termscp
            ;;

            *)
                cargo install termscp
            ;;

        esac
    else
        error "$err"
        error "Alternatively you can opt for installing Cargo <https://www.rust-lang.org/tools/install>"
        return 1
    fi
}

# defaults
if [ -z "${PLATFORM-}" ]; then
    PLATFORM="$(detect_platform)"
fi

if [ -z "${BIN_DIR-}" ]; then
    BIN_DIR=/usr/local/bin
fi

if [ -z "${ARCH-}" ]; then
    ARCH="$(detect_arch)"
fi

if [ -z "${BASE_URL-}" ]; then
    BASE_URL="https://github.com/starship/starship/releases"
fi

# parse argv variables
while [ "$#" -gt 0 ]; do
    case "$1" in
        
        -V | --verbose)
            VERBOSE=1
            shift 1
        ;;
        -f | -y | --force | --yes)
            FORCE=1
            shift 1
        ;;
        -V=* | --verbose=*)
            VERBOSE="${1#*=}"
            shift 1
        ;;
        -f=* | -y=* | --force=* | --yes=*)
            FORCE="${1#*=}"
            shift 1
        ;;
        
        *)
            error "Unknown option: $1"
            exit 1
        ;;
    esac
done

printf "  %s\n" "${UNDERLINE}Termscp configuration${NO_COLOR}"
info "${BOLD}Platform${NO_COLOR}:      ${GREEN}${PLATFORM}${NO_COLOR}"
info "${BOLD}Arch${NO_COLOR}:          ${GREEN}${ARCH}${NO_COLOR}"

# non-empty VERBOSE enables verbose untarring
if [ -n "${VERBOSE-}" ]; then
    VERBOSE=v
    info "${BOLD}Verbose${NO_COLOR}: yes"
else
    VERBOSE=
fi

printf "\n"

confirm "Install ${GREEN}termscp ${TERMSCP_VERSION}${NO_COLOR}?"

# Installation based on arch
case $PLATFORM in
    "freebsd")
        install_on_bsd
    ;;
    "linux")
        install_on_linux
    ;;
    "macos")
        install_on_macos
    ;;
    *)
        error "${PLATFORM} is not supported by this installer"
        exit 1
    ;;
esac

completed "Congratulations! Termscp has successfully been installed on your system!"
info "If you're a new user, you might be interested in reading the user manual <https://veeso.github.io/termscp/#user-manual>"
info "While if you've just updated your termscp version, you can find the changelog at this link <https://veeso.github.io/termscp/#changelog>"
info "Remember that if you encounter any issue, you can report them on Github <https://github.com/veeso/termscp/issues/new>"
info "Feel free to open an issue also if you have an idea which could improve the project"
info "If you want to support the project, please, consider a little donation <https://www.buymeacoffee.com/veeso>"
info "I hope you'll enjoy using termscp :D"

exit 0
