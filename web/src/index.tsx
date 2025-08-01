import { render } from 'solid-js/web'
import './index.css'
import App from './App.tsx'
import { Router } from '@solidjs/router'
import { RootLayout } from './components/RootLayout.tsx'

render(
  () => (
    <Router root={RootLayout}>
      <App />
    </Router>
  ),
  document.getElementById('root')!
)
