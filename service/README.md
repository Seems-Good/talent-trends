# Systemd service to execute:
```bash
docker compose restart on talent-trends
```

# Systemd timer to execute the service @ 2am daily
this is because of a error in logic around getting a OAuth token and can be refactored to avoid needing this in prod.
