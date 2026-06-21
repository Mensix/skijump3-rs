#!/bin/sh
# Enforce single i18n call-site convention:
#   let lang = ...;
#   lang.tr(id)
#
# Disallowed: anything.langbase.tr(, anything.tr( where thing is not "lang"
# Exceptions: fn foo(lang: &LangBase) { lang.tr( ... ) }

set -e
cd "$(git rev-parse --show-toplevel)"

violations=$(rg -n '\.tr\(' game/src -g '*.rs' | \
  # Remove file:line prefix
  sed 's/^[^:]*:[0-9]*://' | \
  # Extract receiver before .tr( (on same line)
  sed -nE 's/.*[^A-Za-z0-9_.()&](([A-Za-z0-9_().&]+)\.tr\().*/\2/p' | \
  # Strip wrapper prefixes
  sed -E 's/^\(//; s/^Some\(//; s/^round_header\(//; s/^&//' | \
  # Keep only non-lang
  grep -v '^lang$' || true)

if [ -n "$violations" ]; then
  echo "ERROR: Non-lang .tr() receivers found:"
  echo "$violations"
  exit 1
fi

echo "OK: all .tr() calls use 'lang' receiver"
