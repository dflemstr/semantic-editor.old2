import * as React from "react";
import { Editor } from "slate-react";

interface Props {
  value: any, // TODO: define schema
}

export default class MainEditor extends React.PureComponent<Props> {
  render() {
    return <Editor value={this.props.value} onChange={(c: any) => {}}/>
  }
}
