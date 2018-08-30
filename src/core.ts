import { SemanticEditor as NativeSemanticEditor } from './wasm/semantic_editor'
import { FileMetadata } from './model'

const NATIVE: Promise<NativeSemanticEditor> = NativeSemanticEditor.new('http://localhost:12345')

export default class SemanticEditor {
  static fetchFileMetadata(path: string): Promise<FileMetadata> {

  }
}
