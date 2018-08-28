import * as React from 'react'
import { Value } from 'slate'
import {
  Alignment, MenuItem, Navbar, NavbarGroup,
  NavbarHeading
} from '@blueprintjs/core'
import '@blueprintjs/core/lib/css/blueprint.css'
import NavbarMenu from './components/NavbarMenu'

interface Props {
}

interface State {
  value: Value
}

class App extends React.Component<Props, State> {
  constructor (props: Props) {
    super(props)
    import('./core').then(editor => {
      console.log('Editor:', editor.default())
    })
    this.state = {
      value: Value.fromJSON({})
    }
  }

  componentDidMount () {
    const value = Value.fromJSON({ document: JSON.parse('{}') })
    this.setState({ value })
  }

  static onChange (change: any) {
  }

  render () {
    return <div>
      <Navbar>
        <NavbarGroup align={Alignment.LEFT}>
          <NavbarMenu title='File'>
            <MenuItem text='New'>
              <MenuItem text='Text file' />
              <MenuItem text='Markdown file' />
            </MenuItem>
            <MenuItem text='Open...' />
          </NavbarMenu>
          <NavbarMenu title='Edit' />
          <NavbarMenu title='View' />
          <NavbarMenu title='Navigate' />
          <NavbarMenu title='Analyze' />
          <NavbarMenu title='Refactor' />
          <NavbarMenu title='Build' />
          <NavbarMenu title='Run' />
          <NavbarMenu title='Tools' />
          <NavbarMenu title='Help' />
        </NavbarGroup>
        <NavbarGroup align={Alignment.RIGHT}>
          <NavbarHeading>Semantic Editor</NavbarHeading>
        </NavbarGroup>
      </Navbar>
    </div>
  }
}

export default App
