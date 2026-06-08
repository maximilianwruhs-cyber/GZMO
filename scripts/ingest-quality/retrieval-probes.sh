#!/usr/bin/env bash
set -eo pipefail

# Dir of this script
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
python3 "$DIR/retrieval-probes.py"
