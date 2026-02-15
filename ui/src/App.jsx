import React, { useState, useEffect } from 'react';
import { useNativeCommand } from './useNativeCommand.js';

export default function App() {
  const sendCommand = useNativeCommand();
  const [pong, setPong] = useState(null);
  const [error, setError] = useState(null);
  const [version, setVersion] = useState(null);
  const [releasesUrl, setReleasesUrl] = useState(null);
  const [updateInfo, setUpdateInfo] = useState(null);
  const [checkingUpdate, setCheckingUpdate] = useState(false);
  const [downloadingUpdate, setDownloadingUpdate] = useState(false);
  const [systemInfo, setSystemInfo] = useState(null);
  const [selectedPath, setSelectedPath] = useState(null);

  useEffect(() => {
    let cancelled = false;
    sendCommand({ name: 'GetVersion' })
      .then((r) => {
        if (cancelled) return;
        if (r?.ok?.version != null) setVersion(r.ok.version);
        if (r?.ok?.releasesUrl != null) setReleasesUrl(r.ok.releasesUrl);
      })
      .catch(() => {});
    return () => { cancelled = true; };
  }, [sendCommand]);

  const onPing = () => {
    setError(null);
    setPong(null);
    sendCommand({ name: 'Ping' })
      .then((r) => setPong(r && r.ok))
      .catch((e) => setError(e?.message));
  };

  const onCheckForUpdates = () => {
    setError(null);
    setUpdateInfo(null);
    setCheckingUpdate(true);
    sendCommand({ name: 'CheckForUpdates' })
      .then((r) => setUpdateInfo(r?.ok ?? null))
      .catch((e) => setError(e?.message ?? 'Failed to check for updates'))
      .finally(() => setCheckingUpdate(false));
  };

  const onOpenReleaseUrl = () => {
    if (updateInfo?.url) {
      sendCommand({ name: 'OpenUrl', url: updateInfo.url }).catch(() => {});
    }
  };

  const onDownloadUpdate = () => {
    if (!updateInfo?.assetUrl) {
      setError('No download URL available');
      return;
    }
    setError(null);
    setDownloadingUpdate(true);
    sendCommand({ name: 'DownloadUpdate', url: updateInfo.assetUrl })
      .then((r) => {
        if (r?.ok?.path) {
          return sendCommand({ name: 'InstallUpdate', path: r.ok.path });
        }
      })
      .catch((e) => setError(e?.message ?? 'Download failed'))
      .finally(() => setDownloadingUpdate(false));
  };

  const onShowAbout = () => {
    setError(null);
    sendCommand({ name: 'GetSystemInfo' })
      .then((r) => setSystemInfo(r?.ok?.info ?? null))
      .catch((e) => setError(e?.message));
  };

  const onOpenFile = () => {
    setError(null);
    setSelectedPath(null);
    sendCommand({
      name: 'OpenFileDialogWithFilters',
      filters: [{ name: 'Text files', extensions: ['txt', 'md'] }],
    })
      .then((r) => r?.ok?.path != null && setSelectedPath(r.ok.path))
      .catch((e) => setError(e?.message));
  };

  const onSaveFile = () => {
    setError(null);
    setSelectedPath(null);
    sendCommand({
      name: 'SaveFileDialog',
      default_name: 'untitled.txt',
      filters: [{ name: 'Text files', extensions: ['txt'] }],
    })
      .then((r) => r?.ok?.path != null && setSelectedPath(r.ok.path))
      .catch((e) => setError(e?.message));
  };

  const onOpenFolder = () => {
    setError(null);
    setSelectedPath(null);
    sendCommand({ name: 'OpenFolderDialog' })
      .then((r) => r?.ok?.path != null && setSelectedPath(r.ok.path))
      .catch((e) => setError(e?.message));
  };

  return (
    <div style={{ padding: 24, fontFamily: 'system-ui', minHeight: '100vh' }}>
      <h1>Desktop Runtime</h1>
      <p>React + Vite (JavaScript). Native bridge via window.native.send.</p>
      {version != null && (
        <p style={{ fontSize: 14, color: '#666' }}>
          Version {version}
          {releasesUrl != null && (
            <>
              {' · '}
              <a
                href="#"
                onClick={(e) => {
                  e.preventDefault();
                  sendCommand({ name: 'OpenUrl', url: releasesUrl }).catch(() => {});
                }}
                style={{ color: '#0066cc' }}
              >
                Release notes
              </a>
            </>
          )}
        </p>
      )}
      {updateInfo?.isNewer && (
        <div
          style={{
            padding: 12,
            marginBottom: 16,
            background: '#e8f5e9',
            borderRadius: 8,
            border: '1px solid #a5d6a7',
          }}
        >
          <strong>Update available:</strong> v{updateInfo.latest}
          {' — '}
          <button type="button" onClick={onOpenReleaseUrl} style={{ marginLeft: 8 }}>
            Open releases page
          </button>
          {updateInfo.assetUrl && (
            <button
              type="button"
              onClick={onDownloadUpdate}
              disabled={downloadingUpdate}
              style={{ marginLeft: 8 }}
            >
              {downloadingUpdate ? 'Downloading…' : 'Download & install'}
            </button>
          )}
        </div>
      )}
      <div style={{ marginBottom: 24 }}>
        <h3 style={{ fontSize: 16 }}>Actions</h3>
        <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <button type="button" onClick={onPing}>
            Ping native
          </button>
          <button
            type="button"
            onClick={onCheckForUpdates}
            disabled={checkingUpdate}
          >
            {checkingUpdate ? 'Checking...' : 'Check for updates'}
          </button>
          <button type="button" onClick={onShowAbout}>
            About / System info
          </button>
          <button type="button" onClick={onOpenFile}>
            Open file
          </button>
          <button type="button" onClick={onSaveFile}>
            Save file
          </button>
          <button type="button" onClick={onOpenFolder}>
            Open folder
          </button>
        </div>
      </div>
      {!updateInfo?.isNewer && updateInfo != null && (
        <p style={{ marginTop: 8, color: '#2e7d32' }}>You have the latest version.</p>
      )}
      {selectedPath != null && (
        <p style={{ fontSize: 14, wordBreak: 'break-all' }}>
          Selected: <code>{selectedPath}</code>
        </p>
      )}
      {systemInfo != null && (
        <details style={{ marginTop: 16 }}>
          <summary>System info</summary>
          <pre style={{ fontSize: 12, overflow: 'auto', marginTop: 8 }}>
            {JSON.stringify(systemInfo, null, 2)}
          </pre>
        </details>
      )}
      {pong != null && <pre>{JSON.stringify(pong)}</pre>}
      {error && <p style={{ color: '#f44336' }}>{error}</p>}
    </div>
  );
}
