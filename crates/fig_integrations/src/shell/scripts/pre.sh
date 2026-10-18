#!/usr/bin/env bash

mkdir -p "${HOME}/.local/bin" > /dev/null 2>&1

# add ~/.local/bin to PATH
if [[ -d "${HOME}/.local/bin" ]] && [[ ":$PATH:" != *":${HOME}/.local/bin:"* ]]; then
  PATH="${PATH:+"$PATH:"}${HOME}/.local/bin"
fi

if [[ -n "${TCOMP_NEW_SESSION:-}" ]]; then
  unset TCOMP_PTY_SESSION_ID
  unset TCOMP_TERM
  unset TCOMP_NEW_SESSION
fi

if [[ -z "${TCOMP_SET_PARENT_CHECK:-}" ]]; then
  # Load parent from env variables
  if [[ -z "${TCOMP_PARENT:-}" && -n "${TCOMP_SET_PARENT:-}" ]]; then
    export TCOMP_PARENT="$TCOMP_SET_PARENT"
    unset -v TCOMP_SET_PARENT
  fi
  export TCOMP_SET_PARENT_CHECK=1
fi

# 0 = Yes, 1 = No, 2 = Fallback to TCOMP_TERM
if [ -z "${SHOULD_TCOMP_PTY_LAUNCH:-}" ]; then
  {{CLI_BINARY_NAME}} _ should-figterm-launch 1>/dev/null 2>&1
  SHOULD_TCOMP_PTY_LAUNCH=$?
fi

# Only launch figterm if current session is not already inside PTY and command exists.
# PWSH var is set when launched by `pwsh -Login`, in which case we don't want to init.
# It is not necessary in Fish.
if   [[ -t 1 ]] \
  && [[ -z "${PROCESS_LAUNCHED_BY_TCOMP:-}" ]] \
  && command -v {{PTY_BINARY_NAME}} 1>/dev/null 2>&1 \
  && [[ ("${SHOULD_TCOMP_PTY_LAUNCH}" -eq 0) || (("${SHOULD_TCOMP_PTY_LAUNCH}" -eq 2) && (-z "${TCOMP_TERM:-}" || (-z "${TCOMP_TERM_TMUX:-}" && -n "${TMUX:-}"))) ]]
then
  # Pty module sets TCOMP_TERM or TCOMP_TERM_TMUX to avoid running twice.
  if [ -z "${TCOMP_SHELL:-}" ]; then
    TCOMP_SHELL=$({{CLI_BINARY_NAME}} _ get-shell)
  fi
  TCOMP_IS_LOGIN_SHELL="${TCOMP_IS_LOGIN_SHELL:='0'}"

  # shellcheck disable=SC2030
  if ([[ -n "${BASH:-}" ]] && shopt -q login_shell) \
    || [[ -n "${ZSH_NAME:-}" && -o login ]]; then
    TCOMP_IS_LOGIN_SHELL=1
  fi

  # Do not launch figterm in non-interactive shells (like VSCode Tasks)
  if [[ $- == *i* ]]; then
    TCOMP_TERM_NAME="$(basename "${TCOMP_SHELL}") ({{PTY_BINARY_NAME}})"
    if [[ -z "${TCOMP_TERM_PATH:-}" ]]; then
      if [[ -x "${HOME}/.local/bin/${TCOMP_TERM_NAME}" ]]; then
        TCOMP_TERM_PATH="${HOME}/.local/bin/${TCOMP_TERM_NAME}"
      else
        TCOMP_TERM_PATH="$(command -v {{PTY_BINARY_NAME}} || echo "${HOME}/.local/bin/{{PTY_BINARY_NAME}}")"
      fi
    fi

    TCOMP_EXECUTION_STRING="${BASH_EXECUTION_STRING:=$ZSH_EXECUTION_STRING}"

    # Get initial text.
    INITIAL_TEXT=""
    # shellcheck disable=SC2031
    if [[ -z "${BASH:-}" || "${BASH_VERSINFO[0]}" -gt "3" ]]; then
      while read -rt 0; do
        if [[ -n "${BASH:-}" ]]; then
          read -r
        fi
        INITIAL_TEXT="${INITIAL_TEXT}${REPLY}\n"
      done
    fi
    TCOMP_EXECUTION_STRING="${TCOMP_EXECUTION_STRING}" TCOMP_START_TEXT="$(printf "%b" "${INITIAL_TEXT}")" TCOMP_SHELL="${TCOMP_SHELL}" TCOMP_IS_LOGIN_SHELL="${TCOMP_IS_LOGIN_SHELL}" exec -a "${TCOMP_TERM_NAME}" "${TCOMP_TERM_PATH}"
  fi
fi
