import { useApp } from '../context/AppContext';

export default function WayInfo() {
  const { state } = useApp();
  const { nearestWay } = state;

  if (!nearestWay) {
    return null;
  }

  return (
    <div className="way-info">
      <div className="way-tags">
        {nearestWay.tags.map((tag, i) => (
          <span key={i} className="way-tag">
            <img src="/icons/tag.svg" alt="Tag" className="icon" />
            {' '}{tag.k} = {tag.v}
          </span>
        ))}
      </div>
      <div className="way-distances">
        <span className="way-distance">
          start: {Math.round(nearestWay.startDistance)} m
        </span>
        <span className="way-distance">
          end: {Math.round(nearestWay.endDistance)} m
        </span>
        <span className="way-distance">
          away: {Math.round(nearestWay.wayDistance)} m
        </span>
      </div>
    </div>
  );
}