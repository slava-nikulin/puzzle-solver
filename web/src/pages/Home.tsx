import { A } from '@solidjs/router'

const puzzles = ['SUDOKU', 'tbd']

export default function Home() {
  return (
    <div class="md:col-start-4 xl:col-start-5">
      <nav class="flex flex-col gap-4 mt-20">
        {puzzles.map((name) => (
          <A
            href={`/${name.toLowerCase()}`}
            class="bg-purple-600 hover:bg-purple-700 text-zinc-200 px-6 py-2 rounded-xl shadow-lg font-semibold transition-colors duration-200"
          >
            {name}
          </A>
        ))}
      </nav>
    </div>
  )
}
