import { Button, Menu, Popover, Position } from "@blueprintjs/core";
import * as React from "react";

interface Props {
  title: string,
}

export default class NavbarMenu extends React.PureComponent<Props> {
  render() {
    return <Popover content={
      <Menu>
        {this.props.children}
      </Menu>} position={Position.BOTTOM_LEFT}>
      <Button className="pt-minimal" text={this.props.title}/>
    </Popover>;
  }
}
