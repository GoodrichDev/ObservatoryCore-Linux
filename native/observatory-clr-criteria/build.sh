#!/usr/bin/env bash
set -euo pipefail
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
dotnet_command=${DOTNET_COMMAND:-dotnet}
publish_dir=${1:?Usage: build.sh /absolute/runtime/output/directory}
runtime_id=${OBSERVATORY_CLR_RUNTIME_ID:-linux-x64}
"$dotnet_command" restore "$script_dir/Observatory.ClrCriteria.csproj" \
  -r "$runtime_id" -p:SelfContained=true --locked-mode
"$dotnet_command" publish "$script_dir/Observatory.ClrCriteria.csproj" \
  -c Release -r "$runtime_id" --self-contained true --no-restore -o "$publish_dir"
python3 "$script_dir/source-provenance.py" "$publish_dir/source-provenance.json"
