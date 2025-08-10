import './App.css'
import { Routes } from './components/Routes'
import { Router } from '@solidjs/router'
import { RootLayout } from './components/RootLayout.tsx'

export default function App() {
  return (
    <Router root={RootLayout}>
      <Routes />
    </Router>
  )
}
