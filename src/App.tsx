import * as React from 'react';
import { Value } from 'slate'
import { Editor } from 'slate-react'
import { Layout, Menu, Icon } from 'antd';
import { wasmBooted, SemanticEditor } from './lib.rs'

const { Sider, Content } = Layout;

let editor: SemanticEditor;

wasmBooted.then(() => {
  editor = SemanticEditor.new();
  editor.init();
}).catch((e: any) => {
  console.error('Failed to load wasm', e);
});

interface Props {
}

interface State {
  value: Value
}

class App extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      value: Value.fromJSON({})
    };
  }

  componentDidMount() {
    wasmBooted.then(() => {
      const value = Value.fromJSON({ document: JSON.parse(editor.document()) });
      this.setState({ value });
    })
  }

  static onChange(change: any) {
    console.log(change.value.toJSON());
  }

  render() {
    return <Layout>
      <Sider style={{ overflow: 'auto', height: '100vh', position: 'fixed', left: 0 }}>
        <div className="logo"/>
        <Menu theme="dark" mode="inline" defaultSelectedKeys={['4']}>
          <Menu.Item key="1">
            <Icon type="user"/>
            <span className="nav-text">nav 1</span>
          </Menu.Item>
          <Menu.Item key="2">
            <Icon type="video-camera"/>
            <span className="nav-text">nav 2</span>
          </Menu.Item>
          <Menu.Item key="3">
            <Icon type="upload"/>
            <span className="nav-text">nav 3</span>
          </Menu.Item>
          <Menu.Item key="4">
            <Icon type="bar-chart"/>
            <span className="nav-text">nav 4</span>
          </Menu.Item>
          <Menu.Item key="5">
            <Icon type="cloud-o"/>
            <span className="nav-text">nav 5</span>
          </Menu.Item>
          <Menu.Item key="6">
            <Icon type="appstore-o"/>
            <span className="nav-text">nav 6</span>
          </Menu.Item>
          <Menu.Item key="7">
            <Icon type="team"/>
            <span className="nav-text">nav 7</span>
          </Menu.Item>
          <Menu.Item key="8">
            <Icon type="shop"/>
            <span className="nav-text">nav 8</span>
          </Menu.Item>
        </Menu>
      </Sider>
      <Layout style={{ marginLeft: 200 }}>
        <Content style={{ overflow: 'initial', backgroundColor: '#fff', padding: '16px' }}>
          <Editor value={this.state.value} onChange={(c: any) => App.onChange(c)}/>
        </Content>
      </Layout>
    </Layout>;
  }
}

export default App;
