import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { Button } from "./components/ui/button";

import "./App.css";
import "../app/globals.css";
import { Films } from "./films/Films";

function App() {
  return (
    <div>
      <Films></Films>
    </div>
  );
}

export default App;
