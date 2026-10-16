#!/usr/bin/env bash
# Paste-ready sequence for the credential holder. Read README.md first.
# Every gh call below was prepared 2026-10-16 against UOR-Foundation/LexLean.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)

gh auth status  # must show a logged-in user with write access to UOR-Foundation/LexLean

# --- 0. Recover the branches from the bundle into your own LexLean clone ---
# The bundle carries main + issue-115-pd08 + issue-116-pd09 + pr-82-rebased.
# Clone or update your checkout, then:
#   cd /path/to/your/LexLean
#   git fetch /path/to/this/dir/lexlean-branches.bundle \
#     'refs/heads/issue-115-pd08:refs/heads/issue-115-pd08' \
#     'refs/heads/issue-116-pd09:refs/heads/issue-116-pd09' \
#     'refs/heads/pr-82-rebased:refs/heads/pr-82-rebased'
# Optional integrity check (expected sha1 tip, full hash in bundle):
#   git rev-parse issue-115-pd08 issue-116-pd09
#     4fcb5b4f6…  9f16effae…

# --- 1. Draft PRs for the two R3 entries (branches now local) ---
cd /path/to/your/LexLean   # <- EDIT THIS to your clone before running
git push origin issue-115-pd08
git push origin issue-116-pd09

gh pr create --repo UOR-Foundation/LexLean --draft \
  --base main --head issue-115-pd08 \
  --title "feat(production): register build-time artifact decoding as PD-08 (R3 entry for #115)" \
  --body-file "$here/pr-115-body.md"

gh pr create --repo UOR-Foundation/LexLean --draft \
  --base main --head issue-116-pd09 \
  --title "feat(production): register kernel-checked overflow discharge as PD-09 (R3 entry for #116)" \
  --body-file "$here/pr-116-body.md"

# --- 2. Claim comments on the issues (post BEFORE review requests) ---
gh issue comment 115 --repo UOR-Foundation/LexLean --body-file "$here/claim-115.md"
gh issue comment 116 --repo UOR-Foundation/LexLean --body-file "$here/claim-116.md"

# --- 3. Decision nudges (D2/D3/D4/D7; 7-day window to 2026-10-23) ---
# The four comments are sections inside nudge-decisions.md; post each section
# on its issue, verbatim. Edit-before-post is fine; the three asks
# (resolved record / delegation / deferral) must survive.

# --- 4. Blocked-lane nudges ---
# Template A from nudge-blocked.md -> #119 #120 #121 #122 #123 #124 #125
# Template B from nudge-blocked.md -> #127

# --- 5. Escalations (3-day and 7-day horizons) ---
gh pr comment 68 --repo UOR-Foundation/LexLean --body-file "$here/nudge-68.md"
# Escalation halves for #91 and #92 are sections in escalation-91-92.md.

# --- 6. #111 deferral record ---
gh issue comment 111 --repo UOR-Foundation/LexLean --body-file "$here/deferral-111.md"

echo "All credential-gated actions must now be verified against the levers' metrics:"
echo "  - both PRs open as draft; claim comments present on #115/#116"
echo "  - decision comments on #107-#110; blocked-lane comments on #119-#125, #127"
echo "  - PR #68 status comment; escalation comments on #91/#92"
echo "  - deferral record on #111"
