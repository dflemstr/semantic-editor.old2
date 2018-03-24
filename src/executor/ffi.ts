import { Microtask } from '../wasm/semantic_editor';

let timeoutId: number;

export function scheduleMicrotask(microtask: Microtask) {
  timeoutId = window.setTimeout(() => {
    clearTimeout(timeoutId);
    if (microtask.run()) {
      microtask.free();
    } else {
      scheduleMicrotask(microtask);
    }
  }, 0);
}
