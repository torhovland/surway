import { useApp } from '../context/AppContext';

export default function EditNoteModal() {
  const { state, setRoute, dispatch, saveEditingNote } = useApp();
  const isNew = state.route === 'newNote';

  return (
    <div className="modal-overlay" onClick={() => {
      dispatch({ type: 'SET_EDITING_NOTE', note: null });
      setRoute('notes');
    }}>
      <div className="modal modal-edit" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{isNew ? 'New Note' : 'Edit Note'}</h3>
          <button className="btn btn-clear float-right" onClick={() => {
            dispatch({ type: 'SET_EDITING_NOTE', note: null });
            setRoute('notes');
          }}>
            &times;
          </button>
        </div>

        <div className="modal-body">
          <textarea
            className="note-textarea"
            placeholder="Enter your note..."
            value={state.editingText}
            onChange={(e) =>
              dispatch({ type: 'SET_EDITING_TEXT', text: e.target.value })
            }
            autoFocus
          />
        </div>

        <div className="modal-footer">
          <button className="btn btn-primary" onClick={saveEditingNote}>
            Save
          </button>
          <button className="btn btn-link" onClick={() => {
            dispatch({ type: 'SET_EDITING_NOTE', note: null });
            setRoute('notes');
          }}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}