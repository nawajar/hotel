# Hotel Management System

Single-property internal staff tool. This phase: scaffold + authentication +
admin-editable UI translations (no bookings/rooms/guests yet).

## Stack

- **Backend**: Rust, Axum, SQLx (compile-time checked queries), PostgreSQL
- **Frontend**: Vue 3 + TypeScript + Vite (Composition API)
- **Auth**: session-based (tower-sessions + Postgres store), argon2 password hashing. No JWT.
- **State/data**: Pinia (auth store), TanStack Vue Query (server calls)
- **UI**: PrimeVue (unstyled) + Tailwind CSS
- **i18n**: vue-i18n, English + Lao, admin-overridable per label

## Translations (English / Lao)

Every label in the frontend is a `t('some.key')` lookup against the bundled default
catalogs in `frontend/src/locales/{en,lo}.json`. Those defaults are the fallback and
also the full registry of valid keys.

An admin can override any individual key/locale pair from **Admin → Translations**
(`/admin/translations`) without touching code or redeploying:

- `GET /api/translations` — public (even the login page needs labels before any
  session exists); returns all saved overrides as `{ locale: { key: value } }`.
- `GET/PUT/DELETE /api/admin/translations` — admin-only CRUD on individual overrides,
  stored in the `translation_overrides` table (`key`, `locale`, `value`). Deleting a
  row reverts that key/locale back to the bundled default.

On boot, the frontend fetches the saved overrides and merges them on top of the
default catalogs; saving/resetting in the admin UI also applies live without a reload.
A language switcher (EN / ລາວ) sits in the top nav, persisted in `localStorage`.

## Running it

```sh
cp .env.example .env   # already done if you're reading this from a fresh clone
docker compose up
```

This brings up, in order:

1. `db` — Postgres 16, waits for `pg_isready`.
2. `seed` — runs migrations, then inserts the seeded admin/employee users
   (idempotent — safe to re-run).
3. `backend` — runs migrations, then `cargo watch -x run` on `:3000`.
4. `frontend` — Vite dev server on `:5173`, proxying `/api` to the backend.

No manual steps required. Open **http://localhost:5173**.

## Logging in

Seeded from `.env` (`ADMIN_EMAIL`/`ADMIN_PASSWORD`, `EMPLOYEE_EMAIL`/`EMPLOYEE_PASSWORD`):

- Admin: `owner@hotel.test` / `changeme-admin` — can reach `/admin`.
- Employee: `staff@hotel.test` / `changeme-staff` — can reach `/dashboard` only;
  `/admin` and `GET /api/admin/ping` return 403/redirect.

There is no public registration — additional employees are added by an admin later
(not part of this phase).

## Hot reload

- Backend: `./backend` is bind-mounted into the `backend`/`seed` containers; `cargo-watch`
  recompiles on save. `/app/target` and the cargo registry are named volumes so rebuilds
  are incremental, not from scratch.
- Frontend: `./frontend` is bind-mounted; Vite HMR picks up changes immediately.
  `node_modules` is an anonymous volume so the host's (possibly absent/incompatible)
  `node_modules` doesn't shadow the one installed in the image.

## Database

- `users`: `id` (uuid pk), `email` (citext, unique), `name`, `password_hash`, `role`
  (`'admin'` | `'employee'`), `created_at`, `updated_at`.
- A second migration creates the schema/table tower-sessions' Postgres store expects
  (`tower_sessions.session`), so `sqlx migrate run` owns the whole schema.

## SQLx: dev vs. prod

In dev, `DATABASE_URL` points at the live `db` service over the compose network, so the
`sqlx::query!`/`query_as!` macros type-check against the real schema at compile time —
**not** offline mode.

For prod builds (no live database available, e.g. CI or a Docker build off the network),
use the committed query cache instead:

```sh
cd backend
cargo sqlx prepare   # regenerate .sqlx/ after changing any query
SQLX_OFFLINE=true cargo build --release
```

Commit the `.sqlx/` directory whenever queries change.

## Prod notes

- `backend/Dockerfile` has a `runtime` stage (cargo-chef cached build → slim Debian image)
  that isn't used by `docker compose up` (which builds the `dev` stage). Build it directly
  with `docker build --target runtime ./backend`.
- `frontend/Dockerfile` has a `prod` stage (`vite build` → nginx) for the same reason.
- Session cookies are issued with `Secure=false` in dev (no TLS on localhost). Flip this to
  `true` in `backend/src/main.rs` once the app is served over HTTPS.
- `SESSION_SECRET` in `.env.example` is a placeholder — generate a real one
  (`openssl rand -hex 64`) for any non-local deployment.

## Project layout

```
backend/    Axum app, SQLx migrations, seed binary, Dockerfile
frontend/   Vue app, Dockerfile
docker-compose.yml
.env.example
```
