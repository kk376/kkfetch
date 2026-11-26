# Bash completion script for kkfetch

_kkfetch() {
    local cur prev words cword
    if declare -F _init_completion >/dev/null 2>&1; then
        _init_completion -n : 2>/dev/null
    else
        if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
            cur="$2"
        else
            cur="${COMP_WORDS[COMP_CWORD]}"
        fi
        prev="$3"
    fi

    local opts="-m -d -l -h -V --modules --disable --no-color --logo --logo-variant --compact-logo --compact --no-logo --list-modules --disk-path --all-disks --json --timings --no-plugins --doctor --help --version"
    local modules="title os host kernel installed install uptime packages pkgs shell display resolution screen wm windowmanager wmtheme wm_theme desktop de terminal term terminalfont termfont term_font theme gtk gtktheme icons icontheme font fonts gtkfont cursor cursortheme mouse cpu gpu memory mem swap disk battery bat localip local_ip ip plugin plugins custom colors palette"
    local logos="ferris rust kkfetch debian ubuntu linuxmint mint kubuntu zorin fedora arch archlinux rhel redhat centos rocky rockylinux almalinux alma endeavouros endeavour manjaro cachyos garuda blackarch omarchy generic tux linux opensuse suse alpine gentoo void pop popos nixos nix kali parrot freebsd openbsd netbsd slackware artix nobara bazzite vylen windows11 windows10 windows7 windows android termux macos darwin none"

    case "${prev}" in
        --logo-variant)
            COMPREPLY=( $(compgen -W "auto full compact" -- "$cur") )
            return 0
            ;;
        -m|--modules|-d|--disable)
            local prefix=""
            local item="$cur"
            if [[ "$cur" == *,* ]]; then
                prefix="${cur%,*},"
                item="${cur##*,}"
            fi
            local matches
            matches=$(compgen -W "$modules" -- "$item")
            COMPREPLY=()
            for m in $matches; do
                COMPREPLY+=("${prefix}${m}")
            done
            return 0
            ;;
        -l|--logo)
            COMPREPLY=( $(compgen -W "$logos" -- "$cur") )
            return 0
            ;;
        --disk-path)
            COMPREPLY=( $(compgen -d -- "$cur") )
            return 0
            ;;
    esac

    if [[ "$cur" == -* || ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
        return 0
    fi
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _kkfetch -o nosort -o bashdefault -o default kkfetch
else
    complete -F _kkfetch -o bashdefault -o default kkfetch
fi
