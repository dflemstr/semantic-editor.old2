import { Microtask } from '../wasm/semantic_editor';

export function scheduleMicrotask(microtask: Microtask) {
  window.setTimeout(() => {
    if (microtask.run()) {
      microtask.free();
    } else {
      scheduleMicrotask(microtask);
    }
  }, 0);
}
