command mkdir -p ~/.local/bin >/dev/null

builtin contains $HOME/.local/bin $PATH
or set --append PATH $HOME/.local/bin

if not test -z "$TCOMP_NEW_SESSION"
    set --erase TCOMP_PTY_SESSION_ID
    set --erase TCOMP_TERM
    set --erase TCOMP_NEW_SESSION
end

# Load parent from env variables
if test -n "$TCOMP_PARENT"; and test -z "$TCOMP_SET_PARENT"
    set --export TCOMP_PARENT $TCOMP_SET_PARENT
end

if test -z "$SHOULD_TCOMP_PTY_LAUNCH"
    # 0 = Yes, 1 = No, 2 = Fallback to TCOMP_TERM
    {{CLI_BINARY_NAME}} _ should-figterm-launch 1>/dev/null 2>&1
    set SHOULD_TCOMP_PTY_LAUNCH $status
end

if test -t 1
    and test -z "$PROCESS_LAUNCHED_BY_TCOMP"
    and command -v {{PTY_BINARY_NAME}} 1>/dev/null 2>/dev/null
    and test "$SHOULD_TCOMP_PTY_LAUNCH" -eq 0 -o \( "$SHOULD_TCOMP_PTY_LAUNCH" -eq 2 -a \( -z "$TCOMP_TERM" -o \( -z "$TCOMP_TERM_TMUX" -a -n "$TMUX" \) \) \)

    if test -z "$TCOMP_SHELL"
        set TCOMP_SHELL ({{CLI_BINARY_NAME}} _ get-shell)
    end
    set TCOMP_IS_LOGIN_SHELL 0
    if status --is-login
        set TCOMP_IS_LOGIN_SHELL 1
    end

    # Do not launch {{PTY_BINARY_NAME}} in non-interactive shells (like VSCode Tasks)
    if status --is-interactive
        set TCOMP_TERM_NAME (command basename "$TCOMP_SHELL")" ({{PTY_BINARY_NAME}})"
        if not set -q TCOMP_TERM_PATH
            if test -x "$HOME/.local/bin/$TCOMP_TERM_NAME"
                set TCOMP_TERM_PATH "$HOME/.local/bin/$TCOMP_TERM_NAME"
            else
                set TCOMP_TERM_PATH (command -v {{PTY_BINARY_NAME}} || echo "$HOME/.local/bin/{{PTY_BINARY_NAME}}")
            end
        end

        # Need to exec bash because we're using 'exec -a <name>'
        # to set argv[0] and fish's exec doesn't have this option
        exec bash -c "TCOMP_PARENT=$TCOMP_PARENT TCOMP_SHELL=$TCOMP_SHELL TCOMP_IS_LOGIN_SHELL=$TCOMP_IS_LOGIN_SHELL exec -a \"$TCOMP_TERM_NAME\" \"$TCOMP_TERM_PATH\""
    end
end
