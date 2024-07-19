import { Film } from "@/films/Films";
import { filmAtom } from "@/lib/atoms";
import { invoke } from "@tauri-apps/api/tauri";
import { useAtom } from "jotai";
import { useEffect } from "react";

function Home() {
  const [films, setFilms] = useAtom(filmAtom);

  async function getFilms() {
    try {
      const data: Film[] | null = await invoke("get_all_films");
      if (data) setFilms(data);
      console.log(films);
    } catch (error) {
      console.error("Failed to fetch films:", error);
    }
  }

  useEffect(() => {
    getFilms();
  }, []);

  return (
    <>
      <div>
        {films?.length ? (
          films.map((film) => (
            <div key={film.id}>
              <h2>{film.title}</h2>
              <p>File: {film.file}</p>
              <p>Link: {film.link}</p>
            </div>
          ))
        ) : (
          <b>No films found</b>
        )}
      </div>
    </>
  );
}

export default Home;
