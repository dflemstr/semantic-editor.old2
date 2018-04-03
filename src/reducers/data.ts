import { Action } from "redux";

interface State {

}

const DEFAULT_STATE: State = {};

export default (state: State = DEFAULT_STATE, action: Action) => {
  return state;
}
