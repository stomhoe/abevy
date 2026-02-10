#!/bin/bash
set -euo pipefail

log_file="/home/stefan/abevy/backup.log"
echo "[$(date)] Backup started" >> "$log_file"

repo_dir="/home/stefan/abevy"
cd "$repo_dir" || exit 1

# Update refs from origin, but keep the backup append-only:
# - Create a uniquely-named tag pointing at the current local HEAD
# - Push the tag to origin (no force), so older backups remain reachable
git fetch origin --prune 2>>"$log_file" || echo "[$(date)] FETCH FAILED" >> "$log_file"

timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
backup_tag="backup/${timestamp}"
backup_ref="HEAD"
backup_sha="$(git rev-parse --verify "$backup_ref" 2>>"$log_file")" \
  || { echo "[$(date)] REV-PARSE FAILED ref=$backup_ref" >> "$log_file"; exit 1; }

# Create an annotated tag for durability and traceability.
# If the tag already exists locally, this will fail (and we log it).
git tag -a "$backup_tag" -m "Automated backup of local $backup_ref ($backup_sha) at $timestamp" "$backup_ref" 2>>"$log_file" \
  || echo "[$(date)] TAG FAILED (already exists?) tag=$backup_tag sha=$backup_sha" >> "$log_file"

git push origin "refs/tags/$backup_tag" 2>>"$log_file" \
  || echo "[$(date)] PUSH FAILED tag=$backup_tag sha=$backup_sha" >> "$log_file"

echo "[$(date)] Backup completed (tag=$backup_tag sha=$backup_sha ref=$backup_ref)" >> "$log_file"
