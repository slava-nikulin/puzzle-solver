/* @refresh reload */
import { render } from 'solid-js/web'
import './index.css'
import App from './App.tsx'
import { Router } from "@solidjs/router";

render(
    () => (
        <Router>
            <App />
        </Router>
    ),
    document.getElementById("root")!
);