import React from 'react';

interface AllianceStats {
  alliance_name: string;
  alliance_id?: number;
  member_count: number;
  village_count: number;
  total_population: number;
  average_population_per_village: number;
  population_growth: number;
  growth_percentage: number;
  alliance_link?: string;
}

interface AllianceInfo {
  top_alliances: AllianceStats[];
  total_alliances: number;
}

interface AllianceInfoModalProps {
  allianceInfo: AllianceInfo;
  onClose: () => void;
}

const formatNumber = (num: number): string => {
  return num.toLocaleString();
};

const formatGrowth = (growth: number): string => {
  return growth >= 0 ? `+${formatNumber(growth)}` : formatNumber(growth);
};

const formatPercentage = (percentage: number): string => {
  if (percentage >= 0) {
    return `+${percentage.toFixed(2)}%`;
  } else {
    return `${percentage.toFixed(2)}%`;
  }
};

const getGrowthColor = (growth: number): string => {
  if (growth > 0) return '#4CAF50'; // Green for positive growth
  if (growth < 0) return '#f44336'; // Red for negative growth
  return '#9E9E9E'; // Gray for no growth
};

export const AllianceInfoModal: React.FC<AllianceInfoModalProps> = ({ allianceInfo, onClose }) => {
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content alliance-info-modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>🏛️ Alliance Information</h2>
          <button onClick={onClose} className="close-btn">❌</button>
        </div>
        
        <div className="alliance-info-content">
          <div className="alliance-summary">
            <div className="summary-card">
              <h3>🛡️ Total Alliances</h3>
              <p>{allianceInfo.total_alliances}</p>
            </div>
            <div className="summary-card">
              <h3>🏆 Top 20 Alliances</h3>
              <p>Ranked by Population</p>
            </div>
          </div>

          <div className="alliances-section">
            <h3>📊 Top 20 Biggest Alliances</h3>
            <div className="alliances-table-container">
              <table className="alliances-table">
                <thead>
                  <tr>
                    <th>Rank</th>
                    <th>Alliance</th>
                    <th>Members</th>
                    <th>Villages</th>
                    <th>Total Population</th>
                    <th>Avg Pop/Village</th>
                    <th>Growth</th>
                    <th>Growth %</th>
                  </tr>
                </thead>
                <tbody>
                  {allianceInfo.top_alliances.map((alliance, index) => (
                    <tr key={alliance.alliance_name} className="alliance-row">
                      <td className="rank-cell">#{index + 1}</td>
                      <td className="alliance-name-cell">
                        {alliance.alliance_link ? (
                          <a 
                            href={alliance.alliance_link} 
                            target="_blank" 
                            rel="noopener noreferrer"
                            className="alliance-link"
                          >
                            {alliance.alliance_name}
                          </a>
                        ) : (
                          alliance.alliance_name
                        )}
                      </td>
                      <td className="members-cell">{formatNumber(alliance.member_count)}</td>
                      <td className="villages-cell">{formatNumber(alliance.village_count)}</td>
                      <td className="population-cell">{formatNumber(alliance.total_population)}</td>
                      <td className="avg-pop-cell">{formatNumber(alliance.average_population_per_village)}</td>
                      <td 
                        className="growth-cell" 
                        style={{ color: getGrowthColor(alliance.population_growth) }}
                      >
                        {formatGrowth(alliance.population_growth)}
                      </td>
                      <td 
                        className="growth-percent-cell"
                        style={{ color: getGrowthColor(alliance.population_growth) }}
                      >
                        {formatPercentage(alliance.growth_percentage)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
          
          <div className="alliance-info-footer">
            <p>📈 Growth data compares current population with previous day</p>
            <p>🔄 Data updates daily with server snapshots</p>
          </div>
        </div>
      </div>
    </div>
  );
};
