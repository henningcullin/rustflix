import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { Button } from "./components/ui/button";

import "./App.css";
import "../app/globals.css";

interface Film {
  id: number;
  title: string | null;
  file: string;
  link: string | null;
}

function App() {
  const [films, setFilms] = useState<Film[]>([]);

  async function getFilms() {
    try {
      const data: Film[] | null = await invoke("get_films");
      if (data) {
        setFilms(data);
      }
    } catch (error) {
      console.error("Failed to fetch films:", error);
    }
  }

  return (
    <div>
      <Button onClick={getFilms}>Get Films</Button>
      <div>
        {films.map((film) => (
          <div key={film.id}>
            <h2>{film.title}</h2>
            <p>File: {film.file}</p>
            <p>Link: {film.link}</p>
          </div>
        ))}
        {!films?.length ? <b>No films stored</b> : ""}
      </div>
    </div>
  );
}

export default App;
