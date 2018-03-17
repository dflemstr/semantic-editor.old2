import { SemanticEditor } from "./wasm/semantic_editor";

export function resolveSemanticEditor(resolve: (value: SemanticEditor) => void, semanticEditor: SemanticEditor) {
  resolve(semanticEditor);
}

export function rejectSemanticEditor(reject: (value: string) => void, error: string) {
  reject(error);
}
