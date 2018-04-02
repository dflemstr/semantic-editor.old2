import { SemanticEditor, FileListing } from './wasm/semantic_editor'
import { booted } from './wasm/semantic_editor_bg'

export * from './wasm/semantic_editor';

const init = async (): Promise<SemanticEditor> => {
  await booted;

  const editor = await new Promise<SemanticEditor>((resolve, reject) => SemanticEditor.new('http://localhost:12345', resolve, reject));
  const files = await new Promise<FileListing>((resolve, reject) => editor.list_files("/", resolve, reject));
  try {
    for (let i = 0; i < files.fileLength(); i++) {
      const file = files.file(i);
      try {
        console.log({
          path: file.path(),
          isRegular: file.isRegular(),
          isDirectory: file.isDirectory()
        });
      } finally {
        file.free();
      }
    }
  } finally {
    files.free();
  }
  return editor;
};

export default init;
