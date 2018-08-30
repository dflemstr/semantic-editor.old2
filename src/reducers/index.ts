import { combineReducers } from "redux";
import data from './data';
import fileTree from './file-tree';

export default combineReducers({
  data,
  fileTree
})
