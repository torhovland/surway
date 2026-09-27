import { useApp } from '../context/AppContext';

export default function NotesModal() {
  const { state, setRoute, dispatch, uploadNote, deleteNote } = useApp();

  return (
    <div className="modal-overlay" onClick={() => setRoute('main')}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>Notes</h3>
          <button className="btn btn-clear float-right" onClick={() => setRoute('main')}>
            &times;
          </button>
        </div>

        <div className="modal-body">
          {state.notes.length === 0 && (
            <p className="text-gray">No notes yet. Take one from your current location!</p>
          )}

          {state.notes.map((note) => (
            <div key={note.id} className="card note-card">
              <div className="card-header">
                <div className="card-title">
                  {new Date(note.time).toLocaleString()}
                </div>
                <div className="card-actions">
                  <button
                    className="btn btn-sm"
                    title="Locate"
                    onClick={() => {
                      dispatch({ type: 'SET_TRACK_POSITION', on: false });
                      setRoute('main');
                      // Position panning handled by map controller
                    }}
                  >
                    <img src="/icons/locate.svg" alt="Locate" className="icon" />
                  </button>
                  <button
                    className={`btn btn-sm ${note.uploaded ? 'btn-orange' : ''}`}
                    title="Upload"
                    onClick={() => uploadNote(note.id)}
                    disabled={state.postingNote}
                  >
                    <img src="/icons/upload.svg" alt="Upload" className="icon" />
                  </button>
                  <button
                    className="btn btn-sm"
                    title="Edit"
                    onClick={() => {
                      dispatch({ type: 'SET_EDITING_NOTE', note });
                      setRoute('editNote');
                    }}
                  >
                    <img src="/icons/pen.svg" alt="Edit" className="icon" />
                  </button>
                  <button
                    className="btn btn-sm btn-error"
                    title="Delete"
                    onClick={() => deleteNote(note.id)}
                  >
                    <img src="/icons/trash.svg" alt="Delete" className="icon" />
                  </button>
                </div>
              </div>
              <div className="card-body">
                {note.text}
              </div>
            </div>
          ))}
        </div>

        <div className="modal-footer">
          <button
            className="btn btn-primary"
            onClick={() => setRoute('newNote')}
          >
            Take a note
          </button>
        </div>
      </div>
    </div>
  );
}