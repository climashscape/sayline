import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import Overlay from "./Overlay";
import "./styles.css";

const isOverlay = window.location.hash === "#/overlay";
if (isOverlay) {
  document.documentElement.classList.add("overlay-root");
}

const storedTheme = window.localStorage.getItem("sayline-theme");
const systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
document.documentElement.dataset.theme =
  storedTheme === "light" || storedTheme === "dark"
    ? storedTheme
    : systemDark
      ? "dark"
      : "light";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>{isOverlay ? <Overlay /> : <App />}</React.StrictMode>,
);
