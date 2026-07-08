#!/bin/sh
set -eu

cleanup_home() {
  home_dir="$1"

  [ -n "$home_dir" ] || return 0
  [ "$home_dir" != "/" ] || return 0
  [ -d "$home_dir" ] || return 0

  for app_id in unofficial-messenger-next io.github.whitersun.unofficial-messenger-next; do
    rm -rf "$home_dir/.config/$app_id"
    rm -rf "$home_dir/.cache/$app_id"
    rm -rf "$home_dir/.local/share/$app_id"
  done
}

cleanup_user_data() {
  if [ "${HOME:-}" != "" ]; then
    cleanup_home "$HOME"
  fi

  cleanup_home /root

  if [ -d /home ]; then
    for home_dir in /home/*; do
      cleanup_home "$home_dir"
    done
  fi
}

if [ "${1:-}" = "0" ]; then
  cleanup_user_data
fi
