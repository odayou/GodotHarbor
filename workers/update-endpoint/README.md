# Godot Harbor Update Endpoint

Cloudflare Worker that serves as the update endpoint for Godot Harbor.

## Endpoints

### App Update
```
GET /updates/{target}/{arch}/{current_version}
```

Returns tauri-plugin-updater compatible JSON:
```json
{
  "version": "0.2.0",
  "notes": "Release notes...",
  "pub_date": "2026-04-26T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "...",
      "url": "https://github.com/odayou/GodotHarbor/releases/download/v0.2.0/GodotHarbor_0.2.0_x64-setup.nsis.zip"
    }
  }
}
```

### Hot Update Manifest
```
GET /hot-update/manifest.json
```

Returns hot update manifest JSON.

## Deployment

```bash
cd workers/update-endpoint
npx wrangler deploy
```

## Configuration

Edit `GITHUB_REPO` in `worker.js` to match your GitHub repository (format: `owner/repo`).
