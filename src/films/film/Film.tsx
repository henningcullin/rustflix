import { useParams } from 'react-router-dom';

function Film() {
  const { filmId } = useParams();

  return <b>filmId: {filmId}</b>;
}

export default Film;
