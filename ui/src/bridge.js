/**
 * IPC via window.native.send. One promise per request; 30s timeout; lifecycle cleaned on timeout/resolve.
 */
const IPC_TIMEOUT_MS = 30000;

function uuid() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

export function send(message) {
  return new Promise((resolve, reject) => {
    if (!window.native || typeof window.native.send !== 'function') {
      reject(new Error('Native bridge not available'));
      return;
    }
    let obj;
    try {
      obj = typeof message === 'string' ? JSON.parse(message) : message;
    } catch (e) {
      reject(e);
      return;
    }
    const id = obj.id || uuid();
    if (!obj.id) obj.id = id;
    const msg = typeof message === 'string' ? message : JSON.stringify(obj);

    const timer = setTimeout(() => {
      if (window.__ipcResolve && window.__ipcResolve[id]) {
        delete window.__ipcResolve[id];
        reject(new Error('IPC timeout'));
      }
    }, IPC_TIMEOUT_MS);

    if (!window.__ipcResolve) window.__ipcResolve = {};
    window.__ipcResolve[id] = (result) => {
      clearTimeout(timer);
      delete window.__ipcResolve[id];
      if (result && result.err) reject(new Error(result.err));
      else resolve(result);
    };

    window.native.send(msg);
  });
}
