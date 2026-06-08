#!/usr/bin/env bash
# Dry-run ingest-eval on M4 Batch-2 golden candidates only (~20 files, ~15–25 min).
# See docs/M4_GOLDEN_INVENTORY.md § "Recommended Order for first replay-wave (Batch 2)".

set -eo pipefail
export LC_ALL=C

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(cd "$DIR/../.." >/dev/null 2>&1 && pwd)"
cd "$PROJECT_ROOT"

CORPUS="/home/maximilian-wruhs/Schreibtisch/knowledge/archive/gzmo_obolus"
STAGING="$DIR/.batch2-corpus"
REPORT_PATH="${REPORT_PATH:-scripts/ingest-quality/report-batch2.json}"

BATCH2_FILES=(
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masterROADMAPmd.md
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masterdocsCONCEPTmd.md
  wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMGZMO_soul_merged_newArtifactsArchitektur_und_Implementierung_autonomer_Sy.md
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masterdocsSCIENTIFIC_FOUNDATIONSmd.md
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masterdocsVISUAL_IDENTITYmd.md
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsops_monitoring_agentmd.md
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentssystem_hygiene_agentmd.md
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsqa_testing_agentmd.md
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsrag_db_agentmd.md
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsobservability_watchermd.md
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsdashboard_curator_agentmd.md
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentsstrategy_analystmd.md
  wave_01_gzmo_obolus_drive_cleanTakeoutDriveObolusObolus-masteragentstoken_agentmd.md
  wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesObolus_Extension__Kon.md
  wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolusNotesProjekt_Obulus__Die_Evolution_der_Digitalen_Glhtml.md
  wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesGitHub_-_maximilianwr.md
  wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesGitHub_-_microsoft_vs.md
  wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesQuelltext_code_3html.md
  wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesQuelltext_code_1html.md
  wave_01_gzmo_obolus_notebooklmTakeoutNotebookLMObolus_VS_Codium_Extension__Konzept__ResearchSourcesWebview_API___Visual_.md
)

rm -rf "$STAGING"
mkdir -p "$STAGING"
linked=0
for f in "${BATCH2_FILES[@]}"; do
  src="$CORPUS/$f"
  if [[ -f "$src" ]]; then
    ln -sf "$src" "$STAGING/$f"
    linked=$((linked + 1))
  else
    echo "[!] missing: $f" >&2
  fi
done
echo "[*] batch2 staging: $linked files in $STAGING"

unset CARGO_TARGET_DIR
cargo build --release -p gzmo-cli
RUST_LOG=warn ./target/release/gzmo ingest-eval "$STAGING" > "$REPORT_PATH" 2>>"$DIR/replay-wave-batch2.stderr.log"
echo "[*] report: $REPORT_PATH"
bash "$DIR/gate-report.sh" "$REPORT_PATH"
bash "$DIR/check-contract.sh" "$REPORT_PATH" || true
