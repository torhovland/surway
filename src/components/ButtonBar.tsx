import { useApp } from '../context/AppContext';
import { getOsmAuthUrl } from '../lib/osm';

export default function ButtonBar() {
  const { state, setRoute } = useApp();

  return (
    <div className="button-bar">
      <button className="btn btn-primary" onClick={() => setRoute('notes')}>
        Notes
      </button>

      {state.user ? (
        <span className="user-info">
          {state.user.photo && (
            <img src={state.user.photo} alt="" className="user-avatar" />
          )}
          {state.user.name}
        </span>
      ) : (
        <a
          className="btn"
          href={getOsmAuthUrl()}
          target="_blank"
          rel="noopener noreferrer"
        >
          OSM login
        </a>
      )}
    </div>
  );
}