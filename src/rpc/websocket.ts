import { WebSocketHandler } from '../wasm';
import * as uuid from 'uuid';

export function setWebSocketHandler(webSocket: WebSocket, handlerId: number) {
  const handler = new WebSocketHandler(handlerId);

  webSocket.onclose = e => {
    try {
      handler.onclose(webSocket, e.code, e.reason, e.wasClean);
    } finally {
      webSocket.onclose = _ => {
      };
      webSocket.onerror = _ => {
      };
      webSocket.onmessage = _ => {
      };
      webSocket.onopen = _ => {
      };
      handler.free();
    }
  };
  webSocket.onerror = e => handler.onerror(webSocket);
  webSocket.onmessage = e => handler.onmessage(webSocket, e.data, e.origin);
  webSocket.onopen = e => handler.onopen(webSocket);
}

export function genUuid(): string {
  return uuid.v4();
}
