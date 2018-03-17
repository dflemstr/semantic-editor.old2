import { Microtask } from '../wasm/semantic_editor';

export function scheduleMicrotask(microtask: Microtask) {
  setTimeout(() => {
    try {
      microtask.run();
    } finally {
      microtask.free();
    }
  }, 0);
}
