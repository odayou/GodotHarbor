const GITHUB_REPO = 'odayou/GodotHarbor';

export default {
  async fetch(request) {
    const url = new URL(request.url);
    const path = url.pathname;

    if (path.startsWith('/updates/')) {
      return handleAppUpdate(request, path);
    }

    if (path === '/hot-update/manifest.json') {
      return handleHotUpdateManifest(request);
    }

    return new Response('Not Found', { status: 404 });
  }
};

async function handleAppUpdate(request, path) {
  const parts = path.split('/').filter(Boolean);
  if (parts.length < 4) {
    return new Response('Bad Request', { status: 400 });
  }

  const target = parts[1];
  const arch = parts[2];
  const currentVersion = parts[3];

  try {
    const release = await getLatestRelease();
    if (!release) {
      return new Response(JSON.stringify({ version: currentVersion }), {
        headers: { 'Content-Type': 'application/json' }
      });
    }

    const latestVersion = release.tag_name.replace(/^v/, '');
    const currentSemver = parseSemver(currentVersion);
    const latestSemver = parseSemver(latestVersion);

    if (!latestSemver || !currentSemver || !isNewer(latestSemver, currentSemver)) {
      return new Response(null, { status: 204 });
    }

    const platformKey = `${target}-${arch}`;
    const asset = findAsset(release, platformKey);

    if (!asset) {
      return new Response(null, { status: 204 });
    }

    const sigAsset = findSigAsset(release, asset.name);

    const response = {
      version: latestVersion,
      notes: release.body || '',
      pub_date: release.published_at,
      platforms: {
        [platformKey]: {
          signature: sigAsset ? await fetchSignature(sigAsset.browser_download_url) : '',
          url: asset.browser_download_url
        }
      }
    };

    return new Response(JSON.stringify(response), {
      headers: {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
        'Cache-Control': 'public, max-age=300'
      }
    });
  } catch (error) {
    return new Response(JSON.stringify({ error: error.message }), {
      status: 500,
      headers: { 'Content-Type': 'application/json' }
    });
  }
}

async function handleHotUpdateManifest(request) {
  const manifest = {
    latest_version: '0.1.0',
    min_compatible_app_version: '0.1.0',
    max_compatible_app_version: '0.2.0',
    release_notes: '',
    pub_date: new Date().toISOString(),
    checksum: '',
    download_url: '',
    signature: '',
    files: []
  };

  return new Response(JSON.stringify(manifest), {
    headers: {
      'Content-Type': 'application/json',
      'Access-Control-Allow-Origin': '*',
      'Cache-Control': 'public, max-age=60'
    }
  });
}

async function getLatestRelease() {
  const cache = caches.default;
  const cacheKey = new Request(`https://github-releases-cache/${GITHUB_REPO}`);
  const cached = await cache.match(cacheKey);
  if (cached) return await cached.json();

  const resp = await fetch(`https://api.github.com/repos/${GITHUB_REPO}/releases/latest`, {
    headers: { 'User-Agent': 'GodotHarbor-UpdateEndpoint' }
  });

  if (!resp.ok) return null;

  const release = await resp.json();
  const cacheResp = new Response(JSON.stringify(release), {
    headers: { 'Cache-Control': 'public, max-age=300' }
  });
  await cache.put(cacheKey, cacheResp);

  return release;
}

function findAsset(release, platformKey) {
  const patterns = {
    'windows-x86_64': /_x64-setup\.nsis\.zip$|_x64\.msi\.zip$/,
    'windows-aarch64': /_arm64-setup\.nsis\.zip$|_arm64\.msi\.zip$/,
    'darwin-x86_64': /_x64\.app\.tar\.gz$/,
    'darwin-aarch64': /_aarch64\.app\.tar\.gz$/,
    'linux-x86_64': /_amd64\.AppImage\.tar\.gz$/,
  };
  const pattern = patterns[platformKey];
  if (!pattern) return null;
  return release.assets.find(a => pattern.test(a.name));
}

function findSigAsset(release, assetName) {
  const sigName = assetName + '.sig';
  return release.assets.find(a => a.name === sigName);
}

async function fetchSignature(url) {
  try {
    const resp = await fetch(url);
    if (resp.ok) return await resp.text();
  } catch {}
  return '';
}

function parseSemver(version) {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)/);
  if (!match) return null;
  return { major: parseInt(match[1]), minor: parseInt(match[2]), patch: parseInt(match[3]) };
}

function isNewer(latest, current) {
  if (latest.major !== current.major) return latest.major > current.major;
  if (latest.minor !== current.minor) return latest.minor > current.minor;
  return latest.patch > current.patch;
}
