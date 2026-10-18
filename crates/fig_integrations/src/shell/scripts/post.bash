if [[ -n "${BASH:-}" ]]; then

# add ~/.local/bin to PATH
if [[ -d "${HOME}/.local/bin" ]] && [[ ":$PATH:" != *":${HOME}/.local/bin:"* ]]; then
  PATH="${PATH:+"$PATH:"}${HOME}/.local/bin"
fi

if [[ -z "${TTY:-}" ]]; then
  TTY=$(tty)
fi
export TTY

export SHELL_PID="$$"

TCOMP_LAST_PS1="$PS1"
TCOMP_LAST_PS2="$PS2"
TCOMP_LAST_PS3="$PS3"

if [[ -z "${TCOMP_SHELL:-}" ]]; then
  TCOMP_SHELL=$({{CLI_BINARY_NAME}} _ get-shell)
fi

# Construct Operating System Command.
# shellcheck disable=SC2059
function tcomp_osc { printf "\033]697;$1\007" "${@:2}"; }

function __tcomp_preexec() {
  tcomp_osc "OSCLock=%s" "${TCOMP_PTY_SESSION_ID}"
  tcomp_osc PreExec

  # Reset user prompts before executing a command, but only if it hasn't
  # changed since we last set it.
  if [[ -n "${TCOMP_USER_PS1+x}" && "${PS1}" = "${TCOMP_LAST_PS1}" ]]; then
    TCOMP_LAST_PS1="${TCOMP_USER_PS1}"
    PS1="${TCOMP_USER_PS1}"
  fi
  if [[ -n "${TCOMP_USER_PS2+x}" && "${PS2}" = "${TCOMP_LAST_PS2}" ]]; then
    TCOMP_LAST_PS2="${TCOMP_USER_PS2}"
    PS2="${TCOMP_USER_PS2}"
  fi
  if [[ -n "${TCOMP_USER_PS3+x}" && "${PS3}" = "${TCOMP_LAST_PS3}" ]]; then
    TCOMP_LAST_PS3="${TCOMP_USER_PS3}"
    PS3="${TCOMP_USER_PS3}"
  fi

  _tcomp_done_preexec="yes"
}

function __tcomp_preexec_preserve_status() {
  __tcomp_ret_value="$?"
  __tcomp_preexec "$@"
  __bp_set_ret_value "${__tcomp_ret_value}" "${__bp_last_argument_prev_command:?}"
}

function __tcomp_pre_prompt () {
  __tcomp_ret_value="$?"

  tcomp_osc "OSCUnlock=%s" "${TCOMP_PTY_SESSION_ID}"
  tcomp_osc "Dir=%s" "${PWD}"
  tcomp_osc "Shell=bash"
  tcomp_osc "ShellPath=%s" "${TCOMP_SHELL:-$SHELL}"
  if [[ -n "${WSL_DISTRO_NAME}" ]]; then
    tcomp_osc "WSLDistro=%s" "${WSL_DISTRO_NAME}"
  fi
  tcomp_osc "PID=%d" "$$"
  tcomp_osc "ExitCode=%s" "$__tcomp_ret_value"
  tcomp_osc "TTY=%s" "${TTY}"
  tcomp_osc "Log=%s" "${TCOMP_LOG_LEVEL}"
  tcomp_osc "User=%s" "${USER:-root}"

  if command -v {{CLI_BINARY_NAME}} >/dev/null 2>&1; then
    (command {{CLI_BINARY_NAME}} _ pre-cmd --alias "$(\alias)" > /dev/null 2>&1 &) >/dev/null 2>&1
  fi

  # Work around bug in CentOS 7.2 where preexec doesn't run if you press ^C
  # while entering a command.
  [[ -z "${_tcomp_done_preexec:-}" ]] && __tcomp_preexec ""
  _tcomp_done_preexec=""

  # Reset $?
  __bp_set_ret_value "${__tcomp_ret_value}" "${__bp_last_argument_prev_command}"
}

function __tcomp_post_prompt () {
  __tcomp_ret_value="$?"

  __tcomp_reset_hooks

  # If TCOMP_USER_PSx is undefined or PSx changed by user, update TCOMP_USER_PSx.
  if [[ -z "${TCOMP_USER_PS1+x}" || "${PS1}" != "${TCOMP_LAST_PS1}" ]]; then
    TCOMP_USER_PS1="${PS1}"
  fi
  if [[ -z "${TCOMP_USER_PS2+x}" || "${PS2}" != "${TCOMP_LAST_PS2}" ]]; then
    TCOMP_USER_PS2="${PS2}"
  fi
  if [[ -z "${TCOMP_USER_PS3+x}" || "${PS3}" != "${TCOMP_LAST_PS3}" ]]; then
    TCOMP_USER_PS3="${PS3}"
  fi

  START_PROMPT="\[$(tcomp_osc StartPrompt)\]"
  END_PROMPT="\[$(tcomp_osc EndPrompt)\]"
  # shellcheck disable=SC2086
  # it's already double quoted, dummy
  NEW_CMD="\[$(tcomp_osc NewCmd=${TCOMP_PTY_SESSION_ID})\]"

  # Reset $? first in case it's used in $TCOMP_USER_PSx.
  __bp_set_ret_value "${__tcomp_ret_value}" "${__bp_last_argument_prev_command}"
  PS1="${START_PROMPT}${TCOMP_USER_PS1}${END_PROMPT}${NEW_CMD}"
  PS2="${START_PROMPT}${TCOMP_USER_PS2}${END_PROMPT}"
  PS3="${START_PROMPT}${TCOMP_USER_PS3}${END_PROMPT}${NEW_CMD}"

  TCOMP_LAST_PS1="${PS1}"
  TCOMP_LAST_PS2="${PS2}"
  TCOMP_LAST_PS3="${PS3}"
}

__tcomp_reset_hooks() {
  # Rely on PROMPT_COMMAND instead of precmd_functions because precmd_functions
  # are all run before PROMPT_COMMAND.
  # Set PROMPT_COMMAND to "[
  #   __tcomp_pre_prompt,
  #   ...precmd_functions,
  #   ORIGINAL_PROMPT_COMMAND,
  #   __tcomp_post_prompt,
  #   __bp_interactive_mode
  # ]"
  local existing_prompt_command
  # shellcheck disable=SC2128
  existing_prompt_command="${PROMPT_COMMAND}"
  existing_prompt_command="${existing_prompt_command//__tcomp_post_prompt[;$'\n']}"
  existing_prompt_command="${existing_prompt_command//__tcomp_post_prompt}"
  existing_prompt_command="${existing_prompt_command//__bp_interactive_mode[;$'\n']}"
  existing_prompt_command="${existing_prompt_command//__bp_interactive_mode}"
  __bp_sanitize_string existing_prompt_command "$existing_prompt_command"

  # shellcheck disable=SC2178
  PROMPT_COMMAND=""
  if [[ -n "${existing_prompt_command:-}" ]]; then
        # shellcheck disable=SC2179
      PROMPT_COMMAND+=${existing_prompt_command}$'\n'
  fi;
  # shellcheck disable=SC2179
  PROMPT_COMMAND+=$'__tcomp_post_prompt\n'
  # shellcheck disable=SC2179
  PROMPT_COMMAND+='__bp_interactive_mode'

  if [[ ${precmd_functions[0]} != __tcomp_pre_prompt ]]; then
    for index in "${!precmd_functions[@]}"; do
      if [[ ${precmd_functions[$index]} == __tcomp_pre_prompt ]]; then
        unset -v 'precmd_functions[$index]'
      fi
    done
    precmd_functions=(__tcomp_pre_prompt "${precmd_functions[@]}")
  fi

  if [[ ${preexec_functions[0]} != __tcomp_preexec_preserve_status ]]; then
    for index in "${!preexec_functions[@]}"; do
      if [[ ${preexec_functions[$index]} == __tcomp_preexec_preserve_status ]]; then
        unset -v 'preexec_functions[$index]'
      fi
    done
    preexec_functions=(__tcomp_preexec_preserve_status "${preexec_functions[@]}")
  fi
}

# Ensure that bash-preexec is installed
# even if the user overrides COMMAND_PROMPT
# https://github.com/withfig/fig/issues/888
#
# We also need to ensure Warp is not running
# since they expect any plugins to not include
# it again
if [[ "${TERM_PROGRAM}" != "WarpTerminal" ]]; then
  __bp_install_after_session_init
fi
__tcomp_reset_hooks
if [[ -n "${PROCESS_LAUNCHED_BY_TCOMP:-}" ]]; then
  tcomp_osc DoneSourcing
fi

fi

(command {{CLI_BINARY_NAME}} _ pre-cmd --alias "$(\alias)" > /dev/null 2>&1 &) >/dev/null 2>&1
