import { useEffect, useState } from 'react';
import { Film } from '../Films';
import { invoke } from '@tauri-apps/api/tauri';
import { useParams } from 'react-router-dom';

function EditFilm() {
  const [film, setFilm] = useState<Film>();

  const { filmId } = useParams();

  async function getFilm(id: number) {
    try {
      const data = await invoke('get_film', { id });
      console.log(data);
    } catch (error) {
      console.error('Could not get film', error);
    }
  }

  useEffect(() => {
    if (typeof filmId !== 'string') return;
    const id = parseInt(filmId);
    getFilm(id);
  }, [filmId]);

  return <div></div>;
}

export default EditFilm;
