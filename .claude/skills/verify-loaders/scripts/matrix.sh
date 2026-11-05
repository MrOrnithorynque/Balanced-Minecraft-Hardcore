#!/bin/bash
# Verifies several branches one after the other: branch.sh, then selftest.sh, for each cell. A cell is a triple
# <origin branch or tag> <minecraft version> <label>.
# usage: matrix.sh                                   the cells of branches.txt, next to this script
#        matrix.sh -f <table file>                   the cells of another table (one triple per line, # starts a comment)
#        matrix.sh <branch> <mc version> <label> [<branch> <mc version> <label> ...]
# Summary: $BMH_VERIFY_OUT/summary.txt (two lines per cell: branch.sh's, then selftest.sh's).
# BMH_LOCAL=1 takes local branches instead of origin's (no push needed); the default is origin's.
# Overnight: charger in and lid open (a closed lid sleeps the Mac even under caffeinate -i, which covers idle sleep only,
# and a sleeping Mac suspends every timeout). Afterwards: pmset -g log | grep 'Entering Sleep' over the run's window.
set -u
SCRIPTS=$(cd "$(dirname "$0")" && pwd)
. "$SCRIPTS/lib.sh"
usage() { echo "usage: matrix.sh [-f table] | matrix.sh <origin branch> <minecraft version> <label> [...]" >&2; exit 2; }

[ $# -gt 0 ] || set -- -f "$SCRIPTS/branches.txt"
if [ "$1" = -f ]; then
  [ $# -eq 2 ] && [ -f "$2" ] || usage
  CELLS=()
  while read -r br mcv label rest; do
    case "$br" in ''|'#'*) continue ;; esac
    [ -n "$label" ] || { echo "bad line in $2: $br $mcv" >&2; exit 2; }
    CELLS+=("$br" "$mcv" "$label")
  done < "$2"
  set -- ${CELLS[@]+"${CELLS[@]}"}
fi
{ [ $# -gt 0 ] && [ $(( $# % 3 )) -eq 0 ]; } || usage

while [ $# -gt 0 ]; do
  bash "$SCRIPTS/branch.sh" "$1" "$2" "$3"
  bash "$SCRIPTS/selftest.sh" "$1" "$2" "$3"
  shift 3
done
echo "MATRIX DONE $(date '+%H:%M:%S')"
