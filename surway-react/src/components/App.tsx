import { useEffect } from 'react';
import { AppProvider, useApp } from '../context/AppContext';
import { useGeolocation } from '../hooks/useGeolocation';
import { useRandomWalk } from '../hooks/useRandomWalk';
import MapView from './MapView';
import WayInfo from './WayInfo';
import ButtonBar from './ButtonBar';
import NotesModal from './NotesModal';
import EditNoteModal from './EditNoteModal';
import '../styles/index.css';

function AppInner() {
  const { state, recalculateNearestWay, triggerOsmDownload } = useApp();

  useGeolocation();
  useRandomWalk();

  // Recalculate nearest way when OSM data or position changes
  useEffect(() => {
    recalculateNearestWay();
  }, [state.position, state.osmData, recalculateNearestWay]);

  // Initial OSM download trigger on first position
  useEffect(() => {
    if (state.position && state.osmData.ways.length === 0 && !state.downloading) {
      triggerOsmDownload();
    }
  }, [state.position, state.osmData.ways.length, state.downloading, triggerOsmDownload]);

  return (
    <div className="app-container">
      <MapView />

      <div className="bottom-panel">
        <ButtonBar />
        <WayInfo />
      </div>

      {state.route === 'notes' && <NotesModal />}
      {(state.route === 'editNote' || state.route === 'newNote') && <EditNoteModal />}
    </div>
  );
}

export default function App() {
  return (
    <AppProvider>
      <AppInner />
    </AppProvider>
  );
}