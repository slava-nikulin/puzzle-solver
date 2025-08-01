import { A } from '@solidjs/router'

const puzzles = ['SUDOKU', 'TODO']

export default function Home() {
  return (
    <nav>
      {puzzles.map((name) => (
        <A href={`/${name.toLowerCase()}`}>{name}</A>
      ))}
    </nav>
  )
}
