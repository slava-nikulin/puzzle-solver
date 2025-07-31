import { FaSolidUser } from "solid-icons/fa";
import { Router, Route, A } from "@solidjs/router";
import './App.css';

import Sudoku from "./routes/Sudoku.tsx";

const puzzles = ["SUDOKU", "TODO"];

import type { RouteSectionProps } from "@solidjs/router";


const Layout = (props: RouteSectionProps) => (
  <div class="bg-gradient-to-br from-gray-900 via-gray-800 to-gray-950 text-gray-100">
    <button
      class="absolute top-4 right-4 text-gray-400 hover:text-gray-200 transition text-2xl"
      aria-label="Profile"
    >
      <FaSolidUser />
    </button>

    {props.children}

    {/* Optional: Add a footer here if needed */}
  </div>
);

export default function App() {
  return (
    <Router root={Layout}>
      <Route path="/" component={() => (
        <div class="w-full max-w-md md:max-w-lg lg:max-w-xl xl:max-w-2xl">
          <div class="text-center space-y-2 animate-slide-down">
            <h1 class="text-4xl md:text-5xl font-semibold">Puzzle Solver</h1>
          </div>
          <ul class="mt-12 grid gap-6 w-full grid-cols-1 md:grid-cols-2 lg:grid-cols-2">
            {puzzles.map((name) => (
              <li>
                <A
                  href={`/${name.toLowerCase()}`}
                  class="w-full py-3 px-6 bg-purple-700 rounded-lg shadow-lg uppercase font-medium tracking-wide transform transition-all duration-200 hover:bg-purple-600 hover:scale-105 focus:outline-none flex items-center justify-center"
                  // Built-in active styling
                  activeClass="ring-2 ring-purple-500"
                >
                  {name}
                </A>
              </li>
            ))}
          </ul>
        </div>
      )} />

      <Route path="/sudoku" component={Sudoku} />
      {/* <Route path="/todo" component={Todo} /> */}
    </Router>
  );
}

