import { invoke } from "@tauri-apps/api/tauri";
import { useState } from "react";

interface Film {
  id: number;
  title: string | null;
  file: string;
  link: string | null;
}

export function Films() {
  const [films, setFilms] = useState<Film[]>([]);

  async function getFilms() {
    try {
      const data: Film[] | null = await invoke("get_all_films");
      if (data) setFilms(data);
      console.log(films);
    } catch (error) {
      console.error("Failed to fetch films:", error);
    }
  }

  return (
    <>
      <button onClick={getFilms}>Refresh films</button>
      <div>
        {films.map((film) => (
          <div key={film.id}>
            <h2>{film.title}</h2>
            <p>File: {film.file}</p>
            <p>Link: {film.link}</p>
          </div>
        ))}
      </div>
    </>
  );
}
