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
  local fail_on_http="${4:-true}"
  local expect_class="${5:-2}" # default expect 2xx

  print_section "${name}"
  local out
  out="$(curl -sS -i "${BASE_URL}${path}")"

  assert_status_prefix "$out" "HTTP/.* ${expect_class}"
  assert_header_present "$out" "x-request-id"

  if [[ "${expect_class}" == "4" ]]; then
    local body
    body="$(echo "$out" | sed -n '/^\r\{0,1\}$/,$p')"
    assert_body_contains "$body" "\"code\""
    assert_body_contains "$body" "\"request_id\""
    assert_body_contains "$body" "NOT_FOUND"
  fi

  echo "$out" | sed -n "1,${lines}p"
  echo
  echo
}

hit_post_json() {
  local name="$1"
  local path="$2"
  local payload="$3"
  local lines="${4:-25}"
  local fail_on_http="${5:-true}"
  local expect_class="${6:-2}" # default expect 2xx

  print_section "${name}"

  local out
  if [[ "${fail_on_http}" == "true" ]]; then
    out="$(curl -fsS -i -X POST "${BASE_URL}${path}" \
      -H "Content-Type: ${CONTENT_TYPE_JSON}" \
      -d "${payload}")"
  else
    out="$(curl -sS -i -X POST "${BASE_URL}${path}" \
      -H "Content-Type: ${CONTENT_TYPE_JSON}" \
      -d "${payload}")"
  fi

  assert_status_prefix "$out" "HTTP/.* ${expect_class}"
  assert_header_present "$out" "x-request-id"

  # For invalid requests, ensure error schema fields exist
  if [[ "${expect_class}" == "4" ]]; then
    local body
    body="$(echo "$out" | sed -n '/^\r\{0,1\}$/,$p')"
    assert_body_contains "$body" "\"request_id\""
    assert_body_contains "$body" "\"code\""
  fi

  echo "$out" | sed -n "1,${lines}p"

  echo
  echo
}

assert_header_present() {
  local headers="$1"
  local header_name="$2"

  if ! echo "$headers" | grep -qi "^${header_name}:"; then
    echo "${RED}${BOLD}FAIL:${NC} missing required header: ${header_name}"
    exit 1
  fi
}

assert_status_prefix() {
  local headers="$1"
  local prefix="$2" # e.g. "HTTP/1.1 2" or "HTTP/1.1 4"
  if ! echo "$headers" | head -n 1 | grep -q "${prefix}"; then
    echo "${RED}${BOLD}FAIL:${NC} unexpected status: $(echo "$headers" | head -n 1)"
    exit 1
  fi
}

assert_body_contains() {
  local body="$1"
  local needle="$2"
  if ! echo "$body" | grep -q "$needle"; then
    echo "${RED}${BOLD}FAIL:${NC} response body missing: ${needle}"
    exit 1
  fi
}

#--------- Tests ---------
echo "${BLUE}==> Smoke test base URL: ${BASE_URL}${NC}"
echo
hit_get "/healthz" "/healthz"
hit_get "/readyz"  "/readyz"
hit_get "/metrics" "/metrics" 15

hit_get "GET /does-not-exist (expect 404 JSON error)" "/does-not-exist" \
  25 \
  false \
  4

hit_post_json "POST /api/v1/orders/validate (valid payload)" \
  "/api/v1/orders/validate" \
  "${VALID_PAYLOAD}"

hit_post_json "POST /api/v1/orders/validate (invalid payload)" \
  "/api/v1/orders/validate" \
  "${INVALID_PAYLOAD}" \
  25 \
  false \
  4

echo "${GREEN}✅ Smoke tests completed"