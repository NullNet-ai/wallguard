# React UI Migration Plan

Replace `crates/wg-ui` (Leptos/WASM) with a React SPA built with Vite + TypeScript.
The server continues to embed and serve the compiled assets — only the frontend stack changes.

---

## Stack

| Concern | Choice |
|---------|--------|
| Build tool | Vite |
| Framework | React 19 + TypeScript |
| Routing | React Router v7 |
| Data fetching | TanStack Query v5 |
| Global state | Zustand |
| Components | shadcn/ui (Radix + Tailwind) |
| Terminal | xterm.js v5 (keep existing terminal.js wrapper) |
| RDP video | WebCodecs (keep existing remote_desktop.js) |

---

## Tasks

### 1 — Infrastructure

- [ ] Create `ui/` directory at repo root; scaffold with `npm create vite@latest`
- [ ] Configure `vite.config.ts`: dev proxy → `http://localhost:4444`, output `dist/`
- [ ] Add `tsconfig.json` with strict mode and path aliases
- [ ] Update `Dockerfile`: add Node 22 build stage before the Rust stage; copy `ui/dist/` into server build context
- [ ] Update `docker-compose.yml`: no UI service needed (assets embedded in server binary)
- [ ] Update `.github/workflows/ci.yml`: replace trunk/wasm32 job with `npm ci && npm run build` in `ui/`
- [ ] Update `.github/workflows/release.yml`: add `npm ci && npm run build` before `cargo build` in server image stage
- [ ] Update `crates/wg-server/build.rs` (or `Cargo.toml`): point `rust-embed` at `../../ui/dist/` instead of `../wg-ui/dist/`
- [ ] Remove `crates/wg-ui` from workspace `Cargo.toml` members and default-members
- [ ] Delete `crates/wg-ui/` entirely
- [ ] Update `CLAUDE.md` build commands

---

### 2 — TypeScript Types

Define in `ui/src/types/` — mirroring `wg-shared` Rust types.

- [ ] `auth.ts` — `TokenResponse`, `LoginRequest`
- [ ] `device.ts` — `Device`, `FirewallKind`, `Feature`, `DeviceStatus`, `MonitoringStatus`
- [ ] `failure.ts` — `AgentFailure`, `FailureSeverity`, `FailureCategory`
- [ ] `tunnel.ts` — `TunnelSession`, `TunnelType`, `TunnelStatus`, `TunnelCreatedResponse`
- [ ] `user.ts` — `User`, `Role`, `CreateUserRequest`
- [ ] `install_code.ts` — `InstallationCode`, `InstallationCodeRow`
- [ ] `http_service.ts` — `HttpService`
- [ ] `api.ts` — generic `ApiError`, `PaginatedResponse<T>`

---

### 3 — API Client

Single `ui/src/api/client.ts` base — mirrors Leptos `api/mod.rs`.

- [ ] Base fetch wrapper: relative URLs, auto-attach `Authorization: Bearer` from Zustand auth store, parse JSON errors into typed `ApiError`
- [ ] `api/auth.ts` — `login(email, password)`, `logout()`, `refresh(refreshToken)`
- [ ] `api/devices.ts` — `listDevices()`, `getDevice(id)`, `getDeviceStatus(id)`
- [ ] `api/tunnels.ts` — `openSsh(deviceId)`, `openTty(deviceId)`, `openHttp(deviceId, host, port)`, `openRdp(deviceId, w, h, fps, kbps)`
- [ ] `api/failures.ts` — `listFailures(deviceId, offset, limit, severity?)`
- [ ] `api/http_services.ts` — `listHttpServices(deviceId)`
- [ ] `api/users.ts` — `listUsers()`, `createUser(req)`, `deleteUser(id)`
- [ ] `api/install_codes.ts` — `listInstallCodes()`, `createInstallCode(ttlHours?)`

---

### 4 — Auth

- [ ] `store/auth.ts` — Zustand store: `token`, `setToken()`, `clearToken()`, persist to `localStorage` via `zustand/middleware`
- [ ] `hooks/useAuth.ts` — returns `{ token, isAuthed, login, logout }`
- [ ] `components/PrivateRoute.tsx` — redirects to `/login` when no token
- [ ] Token appended as `?token=` query param for WebSocket URLs (SSH, TTY, RDP)

---

### 5 — Pages

All pages use TanStack Query for data fetching.

- [ ] **Login** (`/login`) — email + password form, calls `login()`, stores token, redirects to `/`
- [ ] **Dashboard** (`/`) — stat cards (connected devices, recent failures, live events); recent devices table; SSE event log (last 20 of 50 stored)
- [ ] **Device List** (`/devices`) — device cards grid; name, status badge, firewall kind, agent version
- [ ] **Device Detail** (`/devices/:id`) — metadata panel; conditional SSH / TTY / RDP buttons (gated on `features[]`); HTTP services list; SSE listener for `http_services_updated` to refetch services
- [ ] **Device Failures** (`/devices/:id/failures`) — paginated table; severity filter; failure row with badge + message + timestamp
- [ ] **Device Tunnels** (`/devices/:id/tunnels`) — if no session: show Open SSH / TTY / RDP buttons; if session active: render `<Terminal>` or `<RemoteDesktop>` based on `type` query param
- [ ] **Users** (`/settings/users`) — users table; add-user form (email, name, role, password); delete button
- [ ] **Install Codes** (`/settings/install-codes`) — generate code form (TTL hours); display new code with copy button; active codes list with expiry

---

### 6 — Components

- [ ] `StatusBadge.tsx` — connected / disconnected / degraded
- [ ] `SeverityBadge.tsx` — warning / error / fatal
- [ ] `RoleBadge.tsx` — owner / admin / operator / viewer
- [ ] `DeviceCard.tsx` — card used in Device List
- [ ] `StatCard.tsx` — metric card used on Dashboard
- [ ] `Pagination.tsx` — prev/next with page info
- [ ] `Terminal.tsx` — mounts xterm.js via `window.wgTerminal.open()` ref; cleans up on unmount
- [ ] `RemoteDesktop.tsx` — mounts canvas; calls `window.wgRemoteDesktop.open()`; PLI button; cleans up on unmount
- [ ] `EventLog.tsx` — scrollable SSE event feed (used on Dashboard)

---

### 7 — Real-time (SSE)

- [ ] `hooks/useServerEvents.ts` — opens `EventSource` at `/api/v1/events` with auth token in query param; emits typed events; cleans up on unmount
- [ ] Dashboard subscribes to generic `message` events → append to event log (cap 50, display 20)
- [ ] Device Detail subscribes to `http_services_updated` events → invalidate `httpServices` query when event payload matches current device ID

---

### 8 — Static Assets (carry over from wg-ui)

- [ ] Copy `crates/wg-ui/public/terminal.js` → `ui/public/terminal.js`
- [ ] Copy `crates/wg-ui/public/remote_desktop.js` → `ui/public/remote_desktop.js`
- [ ] Copy `crates/wg-ui/public/style.css` → keep as baseline; migrate progressively to Tailwind
- [ ] Load xterm.js from CDN in `index.html` (same versions: core 5.3.0, attach 0.9.0, fit 0.8.0)
- [ ] Add `window.wgTerminal` and `window.wgRemoteDesktop` type declarations in `ui/src/global.d.ts`

---

### 9 — Routing

React Router v7 route tree:

```
/login                          (public)
/                               (PrivateRoute)
  ├─ /                          Dashboard
  ├─ /devices                   DeviceList
  ├─ /devices/:id               DeviceDetail
  ├─ /devices/:id/failures      DeviceFailures
  ├─ /devices/:id/tunnels       DeviceTunnels
  ├─ /settings/users            UsersPage
  └─ /settings/install-codes    InstallCodesPage
```

---

### 10 — Cleanup

- [ ] Confirm server serves `index.html` for all non-API routes (SPA fallback) — check `crates/wg-server/src/api/mod.rs`
- [ ] Confirm `rust-embed` glob points at `ui/dist/` and the 404 fallback returns `index.html`
- [ ] Remove any remaining references to `wg-ui`, `trunk`, `wasm32-unknown-unknown` from docs and CI
- [ ] Update `CLAUDE.md` with new dev commands (`npm run dev` for frontend, `cargo build -p wg-server` for backend)
