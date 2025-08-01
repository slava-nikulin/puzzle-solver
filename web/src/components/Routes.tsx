import { Route } from '@solidjs/router'
import Home from '../pages/Home'
import { lazy } from 'solid-js'

const Sudoku = lazy(() => import('../pages/Sudoku'))

export function Routes() {
  return (
    <>
      <Route path="/" component={Home} />
      <Route path="/sudoku" component={Sudoku} />
    </>
  )
}
