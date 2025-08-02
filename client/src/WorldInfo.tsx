import React from 'react';
import {
  Chart as ChartJS,
  ArcElement,
  Tooltip,
  Legend,
} from 'chart.js';
import { Pie } from 'react-chartjs-2';

ChartJS.register(ArcElement, Tooltip, Legend);

interface TribeStats {
  tribe_id: number;
  tribe_name: string;
  village_count: number;
  total_population: number;
}

interface PlayerStats {
  player_name: string;
  village_count: number;
  total_population: number;
  alliance?: string;
}

interface WorldInfo {
  tribe_stats: TribeStats[];
  top_players: PlayerStats[];
  total_villages: number;
  total_population: number;
}

interface WorldInfoModalProps {
  worldInfo: WorldInfo;
  onClose: () => void;
}

const getTribeColor = (tribeId: number): string => {
  const colors: { [key: number]: string } = {
    1: '#ff6b6b', // Romans - Red
    2: '#4ecdc4', // Teutons - Teal
    3: '#45b7d1', // Gauls - Blue
    4: '#96ceb4', // Nature - Green
    5: '#feca57', // Natars - Yellow
    6: '#ff9ff3', // Egyptians - Pink
    7: '#54a0ff', // Huns - Light Blue
  };
  return colors[tribeId] || '#95a5a6';
};

export const WorldInfoModal: React.FC<WorldInfoModalProps> = ({ worldInfo, onClose }) => {
  // Prepare data for the pie chart
  const chartData = {
    labels: worldInfo.tribe_stats.map(tribe => tribe.tribe_name),
    datasets: [
      {
        data: worldInfo.tribe_stats.map(tribe => tribe.total_population),
        backgroundColor: worldInfo.tribe_stats.map(tribe => getTribeColor(tribe.tribe_id)),
        borderColor: worldInfo.tribe_stats.map(tribe => getTribeColor(tribe.tribe_id)),
        borderWidth: 2,
      },
    ],
  };

  const chartOptions = {
    responsive: true,
    plugins: {
      legend: {
        position: 'right' as const,
      },
      tooltip: {
        callbacks: {
          label: function(context: any) {
            const label = context.label || '';
            const value = context.parsed || 0;
            const percentage = ((value / worldInfo.total_population) * 100).toFixed(1);
            return `${label}: ${value.toLocaleString()} (${percentage}%)`;
          }
        }
      }
    },
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content world-info-modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>🌍 World Information</h2>
          <button onClick={onClose} className="close-btn">❌</button>
        </div>
        
        <div className="world-info-content">
          <div className="world-summary">
            <div className="summary-card">
              <h3>🏘️ Total Villages</h3>
              <p>{worldInfo.total_villages.toLocaleString()}</p>
            </div>
            <div className="summary-card">
              <h3>👥 Total Population</h3>
              <p>{worldInfo.total_population.toLocaleString()}</p>
            </div>
          </div>

          <div className="world-info-sections">
            <div className="tribe-section">
              <h3>🏛️ Tribe Distribution</h3>
              <div className="chart-container">
                <Pie data={chartData} options={chartOptions} />
              </div>
              
              <div className="tribe-stats">
                <h4>Detailed Tribe Statistics:</h4>
                <div className="tribe-stats-grid">
                  {worldInfo.tribe_stats.map((tribe) => (
                    <div key={tribe.tribe_id} className="tribe-stat-card">
                      <div 
                        className="tribe-color" 
                        style={{ backgroundColor: getTribeColor(tribe.tribe_id) }}
                      ></div>
                      <div className="tribe-info">
                        <h5>{tribe.tribe_name}</h5>
                        <p>{tribe.village_count.toLocaleString()} villages</p>
                        <p>{tribe.total_population.toLocaleString()} population</p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            <div className="players-section">
              <h3>👑 Top 10 Players by Population</h3>
              <div className="players-list">
                {worldInfo.top_players.map((player, index) => (
                  <div key={player.player_name} className="player-card">
                    <div className="player-rank">#{index + 1}</div>
                    <div className="player-info">
                      <h4>{player.player_name}</h4>
                      <p>Population: {player.total_population.toLocaleString()}</p>
                      <p>Villages: {player.village_count.toLocaleString()}</p>
                      {player.alliance && (
                        <p className="alliance">Alliance: {player.alliance}</p>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
