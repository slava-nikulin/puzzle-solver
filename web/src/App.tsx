import { createSignal } from "solid-js";
import { FaSolidUser } from "solid-icons/fa";
import './App.css'

const puzzles = ["SUDOKU", "TODO"];

export default function App() {
  const [active, setActive] = createSignal<string | null>(null);

  return (
    <div class="flex flex-col items-center justify-center min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-950 text-gray-100 p-4">
      {/* Profile top-right */}
      <div class="w-full flex justify-end">
        <button class="text-gray-400 hover:text-white transition text-4xl animate-fade-in">
          <FaSolidUser />
        </button>
      </div>

      {/* Title / Blurb */}
      <div class="text-center space-y-2 mt-8 animate-slide-down">
        <h1 class="text-4xl font-semibold">Puzzle Solver</h1>
      </div>

      {/* Menu */}
      <ul class="mt-10 flex flex-col space-y-6 w-full max-w-sm">
        {puzzles.map((name) => (
          <li>
            <button
              onClick={() => setActive(name)}
              class={`w-full py-3 px-6 bg-purple-700 rounded-lg shadow-lg uppercase font-medium tracking-wide
                      transform transition-all duration-200 hover:bg-purple-600 hover:scale-105 focus:outline-none
                      ${active() === name ? "ring-2 ring-purple-500" : ""}`}
            >
              {name}
            </button>
          </li>
        ))}
      </ul>

    </div>
  );
}
