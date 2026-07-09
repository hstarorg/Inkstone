import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { CanvasSpike } from "./spike/canvas-spike";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  window.location.hash === "#spike" ? (
    <CanvasSpike />
  ) : (
    <React.StrictMode>
      <App />
    </React.StrictMode>
  ),
);
