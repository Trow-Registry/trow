#!/usr/bin/env bash
# Enforces the layering contract described in docs/ARCHITECTURE.md.
#
#   routes        -> services
#   services      -> repositories, file_storage, utils
#   repositories  -> nothing above it
#
# `#[cfg(test)]` modules are ignored: tests are allowed to reach for whatever
# fixture they need.
set -uo pipefail

cd "$(dirname "$0")/.." || exit 1

status=0

fail() {
    echo "layering violation: $1" >&2
    echo "$2" >&2
    echo >&2
    status=1
}

# Strip everything from a `#[cfg(test)] mod tests` block onwards, so only
# production code is inspected.
prod_only() {
    awk '
        FNR == 1 { stop = 0 }
        /^#\[cfg\(test\)\]$/ { stop = 1 }
        !stop { print FILENAME ":" FNR ":" $0 }
    ' "$@"
}

# 1. Routes must not touch persistence. Data access belongs to services, and
#    test fixtures belong in src/test_utilities.rs.
out=$(prod_only src/routes/*.rs src/routes/**/*.rs |
    rg -n 'crate::repositories|crate::file_storage' || true)
[ -z "$out" ] || fail "src/routes must not access repositories or file_storage" "$out"

out=$(rg -n 'sqlx::query' src/routes/*.rs src/routes/**/*.rs || true)
[ -z "$out" ] || fail "src/routes must not contain SQL, including in tests" "$out"

# 1b. The raw repositories/storage accessors only exist for the GC's own
#     internals and for the two fixture modules (src/test_utilities.rs for unit
#     tests, tests/common/ for integration tests). Routes and integration test
#     bodies must go through those fixtures.
out=$(rg -n 'services\.(repos|storage)\(\)' src/routes/*.rs src/routes/**/*.rs \
    tests/*.rs 2>/dev/null || true)
[ -z "$out" ] || fail "use the test fixtures instead of reaching for repos()/storage()" "$out"

# 2. Services are transport agnostic: no axum/hyper bodies, extractors, or
#    response types.
out=$(prod_only src/services/*.rs src/services/**/*.rs |
    rg -n 'use (axum|hyper)::|crate::routes' || true)
[ -z "$out" ] || fail "src/services must not depend on the HTTP layer" "$out"

# 3. Repositories only speak SQL and domain models.
out=$(prod_only src/repositories/*.rs |
    rg -n 'use (axum|hyper)::|crate::services|crate::routes' || true)
[ -z "$out" ] || fail "src/repositories must not depend on services or routes" "$out"

# 4. Utils stay at the bottom of the stack.
out=$(prod_only src/utils/*.rs src/file_storage.rs |
    rg -n 'use crate::(services|routes|repositories)' || true)
[ -z "$out" ] || fail "src/utils must not depend on a higher layer" "$out"

exit "$status"
