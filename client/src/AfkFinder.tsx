import React, { useState } from 'react';

interface AfkVillage {
  village_name: string;
  x: number;
  y: number;
  population: number;
  player_name: string;
  alliance?: string;
  days_without_growth: number;
}

interface AfkSearchParams {
  quadrant: string;
  days: number;
}

interface AfkFinderModalProps {
  onClose: () => void;
  onSearch: (params: AfkSearchParams) => void;
  afkVillages: AfkVillage[];
  loading: boolean;
}

export const AfkFinderModal: React.FC<AfkFinderModalProps> = ({ 
  onClose, 
  onSearch, 
  afkVillages, 
  loading 
}) => {
  const [quadrant, setQuadrant] = useState<string>('NE');
  const [days, setDays] = useState<number>(3);

  const handleSearch = () => {
    onSearch({ quadrant, days });
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content afk-finder-modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>🎯 Find AFK Villages</h2>
          <button onClick={onClose} className="close-btn">❌</button>
        </div>
        
        <div className="afk-search-form">
          <div className="search-parameters">
            <div className="parameter-group">
              <label htmlFor="quadrant-select">Quadrant:</label>
              <select
                id="quadrant-select"
                value={quadrant}
                onChange={(e) => setQuadrant(e.target.value)}
                className="parameter-select"
              >
                <option value="NE">Northeast (NE)</option>
                <option value="SE">Southeast (SE)</option>
                <option value="SW">Southwest (SW)</option>
                <option value="NW">Northwest (NW)</option>
              </select>
            </div>
            
            <div className="parameter-group">
              <label htmlFor="days-input">Days without growth:</label>
              <input
                id="days-input"
                type="number"
                min="1"
                max="10"
                value={days}
                onChange={(e) => setDays(Math.max(1, Math.min(10, parseInt(e.target.value) || 1)))}
                className="parameter-input"
              />
            </div>
            
            <button
              onClick={handleSearch}
              disabled={loading}
              className="search-btn"
            >
              {loading ? '🔄 Searching...' : '🔍 Search AFK Villages'}
            </button>
          </div>
        </div>
        
        <div className="afk-results">
          {loading ? (
            <div className="loading-results">
              <p>🔄 Searching for AFK villages...</p>
            </div>
          ) : afkVillages.length > 0 ? (
            <div className="results-container">
              <h3>Found {afkVillages.length} AFK Village(s) in {quadrant} Quadrant</h3>
              <div className="afk-villages-list">
                {afkVillages.map((village, index) => (
                  <div key={`${village.x}-${village.y}-${index}`} className="afk-village-card">
                    <div className="village-header">
                      <h4>{village.village_name}</h4>
                      <span className="coordinates">({village.x}, {village.y})</span>
                    </div>
                    <div className="village-details">
                      <p><strong>Population:</strong> {village.population.toLocaleString()}</p>
                      <p><strong>Player:</strong> {village.player_name}</p>
                      {village.alliance && (
                        <p><strong>Alliance:</strong> {village.alliance}</p>
                      )}
                      <p><strong>No growth for:</strong> {village.days_without_growth} day(s)</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ) : (
            <div className="no-results">
              <p>No AFK villages found in the {quadrant} quadrant for the past {days} day(s).</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
