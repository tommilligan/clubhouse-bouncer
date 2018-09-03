#!/bin/sh

# This script allows inlining of a simple deployable query in shell scripting
# Usage:
#   ./deployable.sh [<ticket_id> ...]
#   ./deployable.sh 1234 9876
#
# Exit codes:
#   0: is deployable
#   1: is not deployable
#   4: probably a jq parse error (bad server response)
#
# Requires:
#   curl
#   jq
#
# Environment:
#   CLUBHOUSE_BOUNCER_API_KEY=<alphanumeric_secret>
#   CLUBHOUSE_BOUNCER_URL=https://<base_hostname>

set -e

# log to stderr
log (){
  printf "$(date --iso-8601=ns)|clubhouse-bouncer|INFO|%s\n" "$@" >&1
}

# form query and send
QUERY="{\"story_ids\":$(printf '%s\n' "$@" | jq -R . | jq -sc .)}"
log "Sending query; $QUERY"

RESPONSE=$(curl -s -X GET --url "$CLUBHOUSE_BOUNCER_URL/deployable" -d "$QUERY" -H "Authorization: $CLUBHOUSE_BOUNCER_API_KEY")
log "Raw response; $RESPONSE"

# parse response
STATUS=$(printf "%s" "$RESPONSE" | jq ".deployable")
log "All tickets are deployable; $STATUS"

# Return appropriate exit code
[ "$STATUS" = "true" ]

