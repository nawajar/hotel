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

## Running the backend manually in production

`docker compose up` is for local dev only. In production the backend runs as a plain
binary on the host — no Docker involved for the backend. (`db` and `frontend` can be
run however you like; this section only covers the Rust binary.)

1. **Build the release binary** (on the host, or in CI — needs Rust installed, not Docker):

   ```sh
   cd backend
   SQLX_OFFLINE=true cargo build --release --bin hotel-backend
   ```

   This uses the committed `.sqlx/` query cache, so no live database is needed at build
   time. Regenerate that cache whenever a query changes — see "SQLx: dev vs. prod" below.

2. **Copy the binary and migrations to the server**, e.g. `/opt/hotel-backend/`:

   ```
   /opt/hotel-backend/hotel-backend       # from target/release/hotel-backend
   /opt/hotel-backend/migrations/         # from backend/migrations/
   /opt/hotel-backend/.env                # see required vars below
   ```

3. **Apply migrations once per deploy** (needs `sqlx-cli`: `cargo install sqlx-cli`):

   ```sh
   cd /opt/hotel-backend
   DATABASE_URL=postgres://... sqlx migrate run
   ```

4. **Run the binary**, with these environment variables set (see `.env.example`):
   - `DATABASE_URL` — pointing at the production Postgres instance.
   - `SESSION_SECRET` — a real random secret (`openssl rand -hex 64`), not the dev placeholder.
   - `COOKIE_SECURE=true` — required once served over HTTPS, so the session cookie gets
     the `Secure` attribute. Leave `false` only for plain-HTTP testing.

   ```sh
   ./hotel-backend
   ```

   For a long-running service rather than a foreground process, see the systemd unit
   example at `backend/deploy/hotel-backend.service.example` (copy it to
   `/etc/systemd/system/hotel-backend.service`, edit the paths/user, then
   `systemctl enable --now hotel-backend`).

- The seed binary (`./hotel-backend`'s sibling `seed`) is dev/bootstrap-only — it's how
  the *first* admin gets created. Don't wire it into a production restart loop; run it
  once by hand if you need to seed a fresh production database, the same way: build it
  with the same `SQLX_OFFLINE` step (`--bin seed`), then run it once with the production
  `DATABASE_URL`/`ADMIN_EMAIL`/`ADMIN_PASSWORD` set.
- `frontend/Dockerfile` has a `prod` stage (`vite build` → nginx) if you want the frontend
  containerized too; entirely independent of how the backend is run.

## Prod notes

- `backend/Dockerfile` also has a `runtime` stage (cargo-chef cached build → slim Debian
  image) if you'd rather containerize the backend after all — build it with
  `docker build --target runtime ./backend`. Not used by `docker compose up` (dev only)
  or by the manual-run path above.
- `SESSION_SECRET` in `.env.example` is a placeholder — generate a real one
  (`openssl rand -hex 64`) for any non-local deployment.

## Project layout

```
backend/           Axum app, SQLx migrations, seed binary, Dockerfile
backend/deploy/     systemd unit example for the manual production run
frontend/          Vue app, Dockerfile
docker-compose.yml  dev-only orchestration
.env.example
```
