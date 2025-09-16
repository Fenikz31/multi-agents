#!/usr/bin/env bash
set -euo pipefail

# Create M0 issues on a GitLab project using glab.
# Usage:
#   scripts/create_m0_issues.sh [--repo group/subgroup/project] [--milestone "M0 – Config & Doctor"]
# Notes:
#   - --repo is optional. If omitted, glab uses the current git remote (recommended).
# Env:
#   GL_HOST / GITLAB_HOST (optional): self-hosted GitLab URL already configured with `glab auth login`.
# Requires:
#   - glab installed and authenticated (`glab auth status`)
#   - templates in .gitlab/issue_templates/

REPO=""
MILESTONE="M0 – Config & Doctor"
CREATE_LABELS=true
CREATE_MILESTONE=true

while [[ $# -gt 0 ]]; do
  case "$1" in
    -R|--repo)
      REPO="$2"; shift 2;;
    -m|--milestone)
      MILESTONE="$2"; shift 2;;
    --no-create-labels)
      CREATE_LABELS=false; shift;;
    --no-create-milestone)
      CREATE_MILESTONE=false; shift;;
    -h|--help)
      echo "Usage: $0 [--repo <group/project>] [--milestone <title>] [--no-create-labels] [--no-create-milestone]"; exit 0;;
    *) echo "Unknown arg: $1"; exit 2;;
  esac
done

if ! command -v glab >/dev/null 2>&1; then
  echo "Error: glab not found in PATH" >&2
  exit 3
fi

# Check auth (non-fatal if it prints warnings)
if ! glab auth status >/dev/null 2>&1; then
  echo "Warning: glab not authenticated. Run: glab auth login --hostname <your_gitlab_host>" >&2
fi

# Idempotent label creation
REPO_FLAG=()
if [[ -n "$REPO" ]]; then
  REPO_FLAG=(-R "$REPO")
  echo "Using explicit repo: $REPO"
else
  echo "Using current repository inferred by glab (no --repo provided)"
fi

if [[ "$CREATE_LABELS" == "true" ]]; then
  echo "Creating/upserting labels on $REPO ..."
  glab label create "m0" --color "#0366d6" "${REPO_FLAG[@]}" 2>/dev/null || true
  glab label create "type:doc" --color "#a2eeef" "${REPO_FLAG[@]}" 2>/dev/null || true
  glab label create "type:tooling" --color "#c5def5" "${REPO_FLAG[@]}" 2>/dev/null || true
  glab label create "type:test" --color "#d4c5f9" "${REPO_FLAG[@]}" 2>/dev/null || true
  glab label create "area:cli" --color "#bfdadc" "${REPO_FLAG[@]}" 2>/dev/null || true
  glab label create "area:compat" --color "#bfd4f2" "${REPO_FLAG[@]}" 2>/dev/null || true
  glab label create "priority:P1" --color "#d73a4a" "${REPO_FLAG[@]}" 2>/dev/null || true
  glab label create "priority:P2" --color "#fbca04" "${REPO_FLAG[@]}" 2>/dev/null || true
fi

# Create milestone if requested (best-effort)
if [[ "$CREATE_MILESTONE" == "true" ]]; then
  echo "Ensuring milestone exists: $MILESTONE"
  # Try to create; if exists, API will fail and we ignore
  glab api "projects/:id/milestones" -F title="$MILESTONE" -F description="Environment, schemas, doctor" "${REPO_FLAG[@]}" >/dev/null 2>&1 || true
fi

# Helper to create an issue
create_issue() {
  local title="$1"
  local file="$2"
  local labels="$3"
  local milestone="$4"
  if [[ ! -f "$file" ]]; then
    echo "Template not found: $file" >&2
    exit 2
  fi
  local desc
  desc=$(cat "$file")
  echo "Creating: $title"
  glab issue create "${REPO_FLAG[@]}" \
    --title "$title" \
    --description "$desc" \
    --label "$labels" \
    --milestone "$milestone" \
    --yes
}

# Base dir (repo root)
ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
TEMPL="$ROOT_DIR/.gitlab/issue_templates"

# Create the 12 issues
create_issue "M0-01 — Codes d’erreur et timeouts par défaut" \
  "$TEMPL/M0-01-codes-et-timeouts.md" \
  "m0,type:doc,priority:P1" \
  "$MILESTONE"

create_issue "M0-02 — Schémas YAML (project/providers) + schema_version" \
  "$TEMPL/M0-02-schemas-yaml.md" \
  "m0,type:doc,priority:P1" \
  "$MILESTONE"

create_issue "M0-03 — Règles de validation métier et placeholders requis" \
  "$TEMPL/M0-03-validations-et-placeholders.md" \
  "m0,type:tooling,priority:P1" \
  "$MILESTONE"

create_issue "M0-04 — Commande config validate (text|json)" \
  "$TEMPL/M0-04-config-validate.md" \
  "m0,area:cli,priority:P1" \
  "$MILESTONE"

create_issue "M0-05 — Conventions tmux (noms, prérequis)" \
  "$TEMPL/M0-05-conventions-tmux.md" \
  "m0,type:doc,priority:P2" \
  "$MILESTONE"

create_issue "M0-06 — Spéc NDJSON + self-check parser" \
  "$TEMPL/M0-06-spec-ndjson.md" \
  "m0,type:doc,priority:P2" \
  "$MILESTONE"

create_issue "M0-07 — Détecteurs de compatibilité CLIs (capabilities snapshot)" \
  "$TEMPL/M0-07-detecteurs-compat.md" \
  "m0,area:compat,priority:P1" \
  "$MILESTONE"

create_issue "M0-08 — Remédiations et table de compatibilité attendue" \
  "$TEMPL/M0-08-remediations-compat.md" \
  "m0,type:doc,priority:P1" \
  "$MILESTONE"

create_issue "M0-09 — Commande doctor (text|json)" \
  "$TEMPL/M0-09-doctor.md" \
  "m0,area:cli,priority:P1" \
  "$MILESTONE"

create_issue "M0-10 — Exemples project.yaml / providers.yaml" \
  "$TEMPL/M0-10-exemples-yaml.md" \
  "m0,type:doc,priority:P2" \
  "$MILESTONE"

create_issue "M0-11 — Documentation M0 (mise à jour)" \
  "$TEMPL/M0-11-docs-m0.md" \
  "m0,type:doc,priority:P2" \
  "$MILESTONE"

create_issue "M0-12 — Tests M0 (unitaires + fumée)" \
  "$TEMPL/M0-12-tests-m0.md" \
  "m0,type:test,priority:P2" \
  "$MILESTONE"

echo "All M0 issues attempted to be created on $REPO with milestone '$MILESTONE'."