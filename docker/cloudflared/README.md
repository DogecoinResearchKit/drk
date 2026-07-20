# Cloudflare Tunnel — local DRK demos

Expose your local DRK Docker instance at **dogeview.space** while this Mac is running. No router port forwarding required.

```
Visitors → Cloudflare → cloudflared (this Mac) → localhost:7070 → DRK Docker
```

## Prerequisites

- [ ] Domain **dogeview.space** added to your Cloudflare account (nameservers pointed at Cloudflare)
- [ ] Dogecoin Core running with `server=1`
- [ ] DRK indexed and serving (see [docker/README.md](../README.md))
- [ ] [cloudflared](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/) installed:

  ```bash
  brew install cloudflared
  ```

## One-time setup

Run from this directory:

```bash
cd docker/cloudflared
./setup.sh
```

### 1. Log in to Cloudflare

```bash
cloudflared tunnel login
```

Opens a browser — pick the **dogeview.space** zone.

### 2. Create a tunnel

```bash
cloudflared tunnel create dogeview-demo
```

Note the **UUID** printed (also the `.json` filename under `~/.cloudflared/`).

To list tunnels later:

```bash
cloudflared tunnel list
```

### 3. Fill in `config.yml`

`setup.sh` created `config.yml` from the example. Edit two fields:

| Field | Value |
|-------|-------|
| `tunnel` | UUID from step 2 |
| `credentials-file` | `./credentials.json` (recommended) |

Copy the credentials file into this folder (do not commit it):

```bash
cp ~/.cloudflared/<UUID>.json ./credentials.json
```

Hostnames in `ingress` default to `dogeview.space` / `www.dogeview.space`. Change them if you prefer a subdomain (e.g. `demo.dogeview.space`).

### 4. Route DNS (once)

```bash
./route-dns.sh
```

Uses `TUNNEL_NAME` and `HOSTNAME` from `.env` (defaults: `dogeview-demo`, `dogeview.space`).

Or manually:

```bash
cloudflared tunnel route dns dogeview-demo dogeview.space
cloudflared tunnel route dns dogeview-demo www.dogeview.space
```

### 5. Start DRK

```bash
docker compose -f ../docker-compose.yml up -d
curl http://localhost:7070/health   # expect 200
```

## Demo day

With Dogecoin Core, DRK, and indexing complete:

```bash
cd docker/cloudflared
./start.sh
```

Share **https://dogeview.space**. Stop the tunnel with `Ctrl+C` when done.

## Files

| File | Purpose |
|------|---------|
| `config.yml.example` | Template — copy to `config.yml` |
| `config.yml` | Your tunnel config (**gitignored**) |
| `credentials.json` | Tunnel secret from Cloudflare (**gitignored**) |
| `.env.example` | `TUNNEL_NAME` / `HOSTNAME` for `route-dns.sh` |
| `setup.sh` | Creates `config.yml` and `.env` from examples |
| `route-dns.sh` | Points hostnames at the tunnel |
| `start.sh` | Runs the tunnel |

## Troubleshooting

**502 / tunnel up but site broken**
- Confirm DRK: `curl http://localhost:7070/health`
- Confirm Docker port mapping is `7070:3110` in `docker-compose.yml`

**DNS not resolving**
- Check Cloudflare dashboard → DNS for CNAME records to `<UUID>.cfargotunnel.com`
- Wait a few minutes after `./route-dns.sh`

**Tunnel credentials error**
- Re-copy `~/.cloudflared/<UUID>.json` → `./credentials.json`
- Ensure `tunnel:` UUID in `config.yml` matches the credentials file

**Site empty / still indexing**
- DRK serves the web UI only after initial sync finishes. Watch progress: `docker compose -f ../docker-compose.yml logs -f`

## Optional: run tunnel on login

```bash
sudo cloudflared service install
```

Point the service at `docker/cloudflared/config.yml` (see [Cloudflare docs](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/configure-tunnels/local-management/as-a-service/macos/)).

## Security notes

- The tunnel exposes your full public DRK API while running.
- For invite-only demos, add [Cloudflare Access](https://developers.cloudflare.com/cloudflare-one/policies/access/) on the hostname.
- Keep `credentials.json` and `config.yml` out of git (already in `.gitignore`).
