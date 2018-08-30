import { Action } from 'redux'
import { FileMetadata, FileNode } from '../model'
import { isType } from 'typescript-fsa'
import actions from '../actions'

interface FileTree {
  root: FileNode,
}

const DEFAULT_STATE: FileTree = {root: {name: '', metadata: null}}

const setChildAt = (node: FileNode, pathParts: string[], ctor: (name: string) => FileNode): FileNode => {
  if (pathParts.length === 0) {
    throw new Error('Illegal argument; pathParts must not be empty')
  } else if (pathParts.length === 1) {
    return ctor(pathParts[0])
  } else if (node.metadata === null) {
    throw new Error(`Cannot update file tree child; node ${node.name} is not yet fetched`)
  } else {
    const children = node.metadata.children
    const childIndex = children.findIndex(n => n.name === pathParts[0])
    const newChild = setChildAt(children[childIndex], pathParts.slice(1), ctor)
    return {
      ...node,
      metadata: {
        ...node.metadata,
        children: children.splice(childIndex, 1, newChild)
      }
    }
  }
}

const createChildAt = (node: FileNode, pathParts: string[]): FileNode => setChildAt(node, pathParts, name => ({
  name,
  metadata: null
}))

const setMetadataAt = (node: FileNode, pathParts: string[], metadata: FileMetadata): FileNode => setChildAt(node, pathParts, name => ({
  name,
  metadata
}))

const getPathParts = (path: string): string[] => path.split('/')

export default (state: FileTree = DEFAULT_STATE, action: Action) => {
  if (isType(action, actions.fileTree.fetchMetadata.started)) {
    const pathParts = getPathParts(action.payload)
    return {...state, root: createChildAt(state.root, pathParts)}
  }

  if (isType(action, actions.fileTree.fetchMetadata.done)) {
    const pathParts = getPathParts(action.payload.params)
    return {...state, root: setMetadataAt(state.root, pathParts, action.payload.result)}
  }

  if (isType(action, actions.fileTree.fetchMetadata.failed)) {
    // TODO
  }

  return state
}
