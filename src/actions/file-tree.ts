import actionCreatorFactory from 'typescript-fsa'
import { FileMetadata } from '../model'
import { wrapAsyncWorker } from './util'
import SemanticEditor from '../core'

const actionCreator = actionCreatorFactory('semantic-editor/file-tree')

export const fetchMetadata = actionCreator.async<string, FileMetadata, {}>('FETCH_METADATA')
export const fetchMetadataWorker = wrapAsyncWorker(fetchMetadata, path => SemanticEditor.fetchFileMetadata(path))
