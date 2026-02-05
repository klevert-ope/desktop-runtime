/** Returns a stable sendCommand for native IPC (bridge.send). */
import { useCallback } from 'react';
import { send } from './bridge.js';

export function useNativeCommand() {
  const sendCommand = useCallback((command) => send(command), []);
  return sendCommand;
}
