export interface FileNode {
  name: string,
  metadata: FileMetadata | null,
}

export interface FileMetadata {
  children: FileNode[],
  file_type: FileType,
  size: number,
  permissions: FilePermissions,
  modified: Date,
  accessed: Date,
  created: Date,
}

export enum FileType {
  FILE,
  DIR,
  SYMLINK,
}

export interface FilePermissions {
  readonly: boolean,
}
