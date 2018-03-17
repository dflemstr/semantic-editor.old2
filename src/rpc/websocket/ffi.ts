import { WebSocketHandler } from '../../wasm/semantic_editor';

export function setWebSocketHandler(webSocket: WebSocket, handler: WebSocketHandler) {
  webSocket.binaryType = 'arraybuffer';
  webSocket.onclose = e => {
    handler.onclose(e.code, e.reason, e.wasClean);
  };
  webSocket.onerror = e => {
    handler.onerror();
  };
  webSocket.onmessage = e => {
    handler.onmessage(new Uint8Array(e.data), e.origin);
  };
  webSocket.onopen = e => {
    handler.onopen();
  };
}

export function clearWebSocketHandler(webSocket: WebSocket, handler: WebSocketHandler) {
  webSocket.onclose = e => {};
  webSocket.onerror = e => {};
  webSocket.onmessage = e => {};
  webSocket.onopen = e => {};
  handler.free();
}
