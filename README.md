# smn_kiosk

Minimal Raspberry Pi kiosk server extracted from `smn_server_sbs`.

Contains only:
- static file serving from `res/static`
- `GET /board/get`
- `POST /board/set`

Run locally:

```bash
cargo run -- --public --port 8000
```

## Raspberry Pi service

Install the systemd service once from the project root:

```bash
chmod +x tools/*.sh
./tools/install_service.sh
```

This installs `smn_kiosk.service`, builds the release binary, deploys it to
`/opt/smn_kiosk`, enables it at boot, and starts it.

For normal code updates after installation:

```bash
./tools/update.sh
```

`update.sh` synchronizes the current Git branch with `origin`, builds the new
release before touching the running deployment, stages the new binary/assets,
and then restarts the service. If the new deployment fails to start, the
previous `/opt/smn_kiosk` deployment is restored.

To rebuild/redeploy the current local checkout without changing Git state:

```bash
./tools/deploy_service.sh
```

Useful service commands:

```bash
sudo systemctl status smn_kiosk
sudo systemctl restart smn_kiosk
journalctl -u smn_kiosk -f
```

The included `res/static/index.html` is a fullscreen board renderer which polls
`/board/get` once per second.
