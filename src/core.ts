import { SemanticEditor } from './wasm/semantic_editor'

export * from './wasm/semantic_editor'

const init = async (): Promise<SemanticEditor> => {
  const editor = await SemanticEditor.new('http://localhost:12345')
  const files = await editor.list_files('/')
  try {
    for (let i = 0; i < files.fileLength(); i++) {
      const file = files.file(i)
      try {
        console.log({
          path: file.path(),
          isRegular: file.isRegular(),
          isDirectory: file.isDirectory()
        })
      } finally {
        file.free()
      }
    }
  } finally {
    files.free()
  }
  return editor
}

export default init
