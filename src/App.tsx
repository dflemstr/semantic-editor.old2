import * as React from 'react';
import { Value } from 'slate'
import { Editor } from 'slate-react'
import { Layout, Tree, Menu, Input, Icon } from 'antd';
import { SemanticEditor, init as wasmInit } from './wasm/semantic_editor'
import { booted as wasmBooted } from './wasm/semantic_editor_bg'

const { Sider, Content } = Layout;
const { TreeNode } = Tree;
const { Search } = Input;

let editor: SemanticEditor;

let editorBooted = wasmBooted.then(async () => {
  wasmInit();
  editor = await new Promise<SemanticEditor>((resolve, reject) => SemanticEditor.new('wss://echo.websocket.org', resolve, reject));
  editor.send_rpc();
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
    editorBooted.then(() => {
      const value = Value.fromJSON({ document: JSON.parse(editor.document()) });
      this.setState({ value });
    })
  }

  static onChange(change: any) {
  }

  render() {
    return <Layout>
      <Sider collapsible>
        <div style={{ overflowY: 'auto', height: 'calc(100vh - 48px)' }}>
          <div style={{ margin: '8px' }}>
            <Search placeholder='Search...'/>
          </div>
          <Menu theme="dark" mode="inline">
            <Menu.Item key="overview">
              <Icon type="home"/><span>Overview</span>
            </Menu.Item>
            <Menu.Item key="settings">
              <Icon type="setting"/><span>Settings</span>
            </Menu.Item>
          </Menu>
          <div style={{ background: '#fff' }}>
            <Tree showLine>
              <TreeNode title={"foo"} key={1}/>
            </Tree>
          </div>
        </div>
      </Sider>
      <Layout style={{ overflowY: 'auto', height: '100vh' }}>
        <Content style={{ overflow: 'initial', backgroundColor: '#fff', padding: '16px' }}>
          <Editor value={this.state.value} onChange={(c: any) => App.onChange(c)}/>
        </Content>
      </Layout>
    </Layout>;
  }
}

export default App;