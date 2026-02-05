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

  useEffect(() => {
    sendCommand({ name: 'GetVersion' })
      .then((r) => {
        if (r?.ok?.version != null) setVersion(r.ok.version);
        if (r?.ok?.releasesUrl != null) setReleasesUrl(r.ok.releasesUrl);
      })
      .catch(() => {});
  }, [sendCommand]);

  const onPing = () => {
    setError(null);
    setPong(null);
    sendCommand({ name: 'Ping' })
      .then((r) => setPong(r && r.ok))
      .catch((e) => setError(e && e.message));
  };

  const onCheckForUpdates = () => {
    setError(null);
    setUpdateInfo(null);
    setCheckingUpdate(true);
    sendCommand({ name: 'CheckForUpdates' })
      .then((r) => {
        setUpdateInfo(r?.ok ?? null);
      })
      .catch((e) => setError(e?.message ?? 'Failed to check for updates'))
      .finally(() => setCheckingUpdate(false));
  };

  const onOpenReleaseUrl = () => {
    if (updateInfo?.url) {
      sendCommand({ name: 'OpenUrl', url: updateInfo.url }).catch(() => {});
    }
  };

  return (
    <div style={{ padding: 24, fontFamily: 'system-ui' }}>
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
            Download
          </button>
        </div>
      )}
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
      </div>
      {!updateInfo?.isNewer && updateInfo != null && (
        <p style={{ marginTop: 8, color: '#2e7d32' }}>You have the latest version.</p>
      )}
      {pong != null && <pre>{JSON.stringify(pong)}</pre>}
      {error && <p style={{ color: 'red' }}>{error}</p>}
    </div>
  );
}
