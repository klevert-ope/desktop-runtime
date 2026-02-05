# React best practices — avoiding memory leaks

Short guide for the UI in `ui/` (React, hooks, native bridge). Prevents memory leaks and "Can't perform a React state update on an unmounted component" warnings.

## 1. useEffect cleanup

When an effect subscribes to something (timers, listeners, async work), return a cleanup function so it runs on unmount or before the effect re-runs.

- **Rule:** If the effect starts a side effect that outlives the render, clean it up.
- React docs: [Effect cleanup](https://react.dev/learn/synchronizing-with-effects#step-3-add-cleanup-if-needed).

## 2. Async and setState after unmount

Any async work (fetch, `sendCommand`, timers) that calls `setState` must not run after the component unmounts. Otherwise React warns and you can leak references.

**Pattern — cancelled flag:**

```javascript
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
```

**Alternative:** Use `AbortController` for fetch and ignore state updates when `signal.aborted` is true.

## 3. Timers

Store timer ids and clear them in cleanup:

```javascript
useEffect(() => {
  const id = setInterval(() => { /* ... */ }, 1000);
  return () => clearInterval(id);
}, []);
```

Same for `setTimeout`: capture the id and call `clearTimeout` in the cleanup.

## 4. Event listeners

Add listeners in the effect, remove in cleanup:

```javascript
useEffect(() => {
  const handler = () => { /* ... */ };
  window.addEventListener('resize', handler);
  return () => window.removeEventListener('resize', handler);
}, []);
```

## 5. Subscriptions and long-lived objects

WebSockets, observers, third-party subscriptions: unsubscribe or disconnect in the effect cleanup. Otherwise the subscription keeps a reference to the component.

## 6. This project’s UI

In [ui/src/App.jsx](../ui/src/App.jsx):

- The `useEffect` that calls `sendCommand({ name: 'GetVersion' })` then `setVersion` / `setReleasesUrl` has **no cleanup**. If the component unmounts before the promise resolves, React will warn. Add a cleanup that sets a cancelled flag and guard the `setState` calls.
- The same pattern applies to any other async path that calls setState (e.g. Ping, CheckForUpdates): either the async work is tied to a user action and the component is usually still mounted, or you should guard setState with a cancelled check from effect cleanup.

Recommendation: add the cancelled-flag pattern to every `useEffect` that triggers async work and then updates state, so unmount (or strict mode double-mount) never calls setState after cleanup.
