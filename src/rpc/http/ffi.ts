import { HttpFetchHandler } from "../../wasm/semantic_editor";

const REQUEST_CONTENT_TYPE = "application/x-semantic-editor-request";
const RESPONSE_CONTENT_TYPE = "application/x-semantic-editor-response";

export function performFetch(url: string, data: Uint8Array, handler: HttpFetchHandler) {
  fetch(url, {
    method: 'POST',
    headers: new Headers({
      "Content-Type": REQUEST_CONTENT_TYPE,
    }),
    mode: 'cors',
    body: data,
  }).then(response => {
    if (response.ok && response.headers.get("Content-Type") === RESPONSE_CONTENT_TYPE) {
      response.arrayBuffer().then((buffer) => {
        try {
          handler.resolve(new Uint8Array(buffer));
        } finally {
          handler.free();
        }
      });
    } else {
      try {
        handler.reject("got bad response: " + response);
      } finally {
        handler.free();
      }
    }
  }).catch((error) => {
    try {
      handler.reject(error.toString());
    } finally {
      handler.free();
    }
  });
}
