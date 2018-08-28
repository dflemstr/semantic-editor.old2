import { FileListing, SemanticEditor } from './wasm/semantic_editor'

export function resolveSemanticEditor (resolve: (value: SemanticEditor) => void, value: SemanticEditor) {
  resolve(value)
}

export function rejectSemanticEditor (reject: (value: string) => void, value: string) {
  reject(value)
}

export function resolveFileListing (resolve: (value: FileListing) => void, value: FileListing) {
  resolve(value)
}

export function rejectFileListing (reject: (value: string) => void, value: string) {
  reject(value)
}
