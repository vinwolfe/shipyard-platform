#!/usr/bin/env bash
set -euo pipefail

#--------- Env ---------
PORT="${SERVICE_PORT:-8080}"
BASE_URL="${BASE_URL:-http://localhost:${PORT}}"

#--------- Payloads ---------
VALID_PAYLOAD='{"external_id":"ord_123","items":[{"sku":"ABC","qty":1}]}'
INVALID_PAYLOAD='{"external_id":"","items":[]}'
CONTENT_TYPE_JSON="application/json"

#--------- Colours ---------
if [[ -t 1 ]]; then
    RED=$'\033[0;31m'
    GREEN=$'\033[0;32m'
    YELLOW=$'\033[0;33m'
    BLUE=$'\033[0;34m'
    BOLD=$'\033[1m'
NC=$'\033[0m'
else
  RED=''; GREEN=''; YELLOW=''; BLUE=''; BOLD=''; NC=''
fi


#--------- Helpers ---------
print_section() {
  local name="$1"
  echo "${YELLOW}---- ${name} ----${NC}"
}

hit_get() {
  local name="$1"
  local path="$2"
  local lines="${3:-10}"

  print_section "${name}"
  curl -fsS -i "${BASE_URL}${path}" | sed -n "1,${lines}p"
  echo
  echo
}

hit_post_json() {
  local name="$1"
  local path="$2"
  local payload="$3"
  local lines="${4:-25}"
  local fail_on_http="${5:-true}"

  print_section "${name}"

  if [[ "${fail_on_http}" == "true" ]]; then
    curl -fsS -i -X POST "${BASE_URL}${path}" \
      -H "Content-Type: ${CONTENT_TYPE_JSON}" \
      -d "${payload}" \
      | sed -n "1,${lines}p"
  else
    curl -sS -i -X POST "${BASE_URL}${path}" \
      -H "Content-Type: ${CONTENT_TYPE_JSON}" \
      -d "${payload}" \
      | sed -n "1,${lines}p"
  fi

  echo
  echo
}

#--------- Tests ---------
echo "${BLUE}==> Smoke test base URL: ${BASE_URL}${NC}"
echo
hit_get "/healthz" "/healthz" 10
hit_get "/readyz"  "/readyz"  10

hit_post_json "POST /api/v1/orders/validate (valid payload)" \
  "/api/v1/orders/validate" \
  "${VALID_PAYLOAD}" \
  25

hit_post_json "POST /api/v1/orders/validate (invalid payload)" \
  "/api/v1/orders/validate" \
  "${INVALID_PAYLOAD}" \
  25 \
  false

echo "${GREEN}✅ Smoke tests completed"