import { useState, useEffect } from 'react'
import './App.css'
import { WorldInfoModal } from './WorldInfo'

interface Village {
  id: number;
  name: string;
  x: number;
  y: number;
  population: number;
  player?: string;
  alliance?: string;
  worldid?: number;
}

interface HealthResponse {
  status: string;
  message: string;
}

interface Server {
  id: number;
  name: string;
  url: string;
  is_active: boolean;
}

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

function App() {
  const [villages, setVillages] = useState<Village[]>([]);
  const [serverStatus, setServerStatus] = useState<HealthResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');
  const [isConnecting, setIsConnecting] = useState(false);
  const [servers, setServers] = useState<Server[]>([]);
  const [currentServer, setCurrentServer] = useState<Server | null>(null);
  const [newServerName, setNewServerName] = useState<string>('');
  const [newServerUrl, setNewServerUrl] = useState<string>('');
  const [showAddServer, setShowAddServer] = useState(false);
  const [notification, setNotification] = useState<string>('');
  const [showWorldInfo, setShowWorldInfo] = useState(false);
  const [worldInfo, setWorldInfo] = useState<WorldInfo | null>(null);
  const [loadingWorldInfo, setLoadingWorldInfo] = useState(false);
  
  const serverUrl = 'http://127.0.0.1:3001'; // Fixed server URL

  useEffect(() => {
    const fetchData = () => {
      setIsConnecting(true);
      setError('');
      
      Promise.all([fetchServerStatus(), fetchVillages(), fetchServers()])
        .finally(() => {
          setIsConnecting(false);
        });
    };

    // Initial load
    fetchData();
  }, []); // No dependencies since serverUrl is now fixed

  const fetchServerStatus = async () => {
    try {
      const response = await fetch(`${serverUrl}/health`);
      if (response.ok) {
        const data = await response.json();
        setServerStatus(data);
        setError(''); // Clear error on success
      } else {
        setServerStatus(null);
        setError('Failed to connect to server');
      }
    } catch (err) {
      setServerStatus(null);
      setError('Server connection error');
      console.error('Health check error:', err);
    }
  };

  const fetchVillages = async () => {
    try {
      // Only show loading on initial load
      if (villages.length === 0) {
        setLoading(true);
      }
      const response = await fetch(`${serverUrl}/api/villages`);
      if (response.ok) {
        const data = await response.json();
        setVillages(data);
        setError(''); // Clear error on success
      } else {
        setError('Failed to fetch villages');
      }
    } catch (err) {
      setError('Failed to fetch data');
      console.error('Fetch error:', err);
    } finally {
      setLoading(false);
    }
  };

  const fetchServers = async () => {
    try {
      const response = await fetch(`${serverUrl}/api/servers`);
      if (response.ok) {
        const data = await response.json();
        setServers(data.servers || []);
        // Set current server to the active one
        const activeServer = data.servers?.find((s: Server) => s.is_active);
        setCurrentServer(activeServer || null);
      } else {
        console.error('Failed to fetch servers');
      }
    } catch (err) {
      console.error('Error fetching servers:', err);
    }
  };

  const addServer = async () => {
    if (!newServerName.trim() || !newServerUrl.trim()) return;
    
    try {
      const response = await fetch(`${serverUrl}/api/servers`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name: newServerName.trim(),
          url: newServerUrl.trim()
        }),
      });

      if (response.ok) {
        setNewServerName('');
        setNewServerUrl('');
        setShowAddServer(false);
        await fetchServers(); // Refresh server list
      } else {
        setError('Failed to add server');
      }
    } catch (err) {
      setError('Failed to add server');
      console.error('Add server error:', err);
    }
  };

  const setActiveServer = async (serverId: number) => {
    try {
      const response = await fetch(`${serverUrl}/api/servers/${serverId}/activate`, {
        method: 'PUT',
      });

      if (response.ok) {
        const result = await response.json();
        
        // Show auto-load message if available
        if (result.auto_load_message) {
          console.log('Auto-load result:', result.auto_load_message);
          setNotification(result.auto_load_message);
          // Clear notification after 5 seconds
          setTimeout(() => setNotification(''), 5000);
        }
        
        await fetchServers(); // Refresh server list
        await fetchVillages(); // Refresh villages for new server
      } else {
        setError('Failed to set active server');
      }
    } catch (err) {
      setError('Failed to set active server');
      console.error('Set active server error:', err);
    }
  };

    const removeServer = async (serverId: number, serverName: string) => {
    try {
      const response = await fetch(`${serverUrl}/api/servers/${serverId}`, {
        method: 'DELETE',
      });

      if (response.ok) {
        setNotification(`Server "${serverName}" removed successfully`);
        // Clear notification after 5 seconds
        setTimeout(() => setNotification(''), 5000);
        
        await fetchServers(); // Refresh server list
        await fetchVillages(); // Refresh villages (might be empty if we removed active server)
      } else {
        setError('Failed to remove server');
      }
    } catch (err) {
      setError('Failed to remove server');
      console.error('Remove server error:', err);
    }
  };

  const fetchWorldInfo = async () => {
    if (!currentServer) {
      setError('No active server selected');
      return;
    }

    try {
      setLoadingWorldInfo(true);
      const response = await fetch(`${serverUrl}/api/world-info`);
      
      if (response.ok) {
        const data = await response.json();
        setWorldInfo(data.data);
        setShowWorldInfo(true);
        setError('');
      } else {
        setError('Failed to fetch world info');
      }
    } catch (err) {
      setError('Failed to fetch world info');
      console.error('World info fetch error:', err);
    } finally {
      setLoadingWorldInfo(false);
    }
  };

  return (
    <div className="App">
      <header className="app-header">
        <h1>🏰 Travian Map Explorer</h1>
        <div className="server-status">
          {isConnecting ? (
            <div className="status-connecting">
              🔄 Connecting to server...
            </div>
          ) : serverStatus ? (
            <div className="status-ok">
              ✅ Server: {serverStatus.message}
            </div>
          ) : (
            <div className="status-error">
              ❌ {error || 'Not connected'}
            </div>
          )}
        </div>
      </header>

      {notification && (
        <div className="notification">
          ℹ️ {notification}
        </div>
      )}

      <main>
        <div className="server-management">
          <div className="current-server">
            <label htmlFor="server-select">Current Server:</label>
            <select
              id="server-select"
              value={currentServer?.id || ''}
              onChange={(e) => {
                const serverId = parseInt(e.target.value);
                if (serverId) setActiveServer(serverId);
              }}
              className="server-select"
            >
              <option value="">Select a server...</option>
              {servers.map((server) => (
                <option key={server.id} value={server.id}>
                  {server.name} ({server.url})
                </option>
              ))}
            </select>
          </div>

          {currentServer && (
            <div className="world-info-section">
              <button 
                onClick={fetchWorldInfo}
                disabled={loadingWorldInfo}
                className="world-info-btn"
              >
                {loadingWorldInfo ? '🔄 Loading...' : '📊 World Info'}
              </button>
            </div>
          )}
          
          <div className="manage-servers-section">
            <h3>Manage Servers:</h3>
            <div className="servers-list">
              {servers.map((server) => (
                <div key={server.id} className="server-item">
                  <div className="server-info">
                    <span className="server-name">
                      {server.name} {server.is_active && '(Active)'}
                    </span>
                    <span className="server-url">{server.url}</span>
                  </div>
                  <button 
                    onClick={() => removeServer(server.id, server.name)}
                    className="remove-server-btn"
                    disabled={servers.length === 1}
                    title={servers.length === 1 ? "Cannot remove the last server" : `Remove ${server.name}`}
                  >
                    🗑️ Remove
                  </button>
                </div>
              ))}
            </div>
          </div>
          
          <div className="add-server-section">
            {!showAddServer ? (
              <button 
                onClick={() => setShowAddServer(true)}
                className="add-server-btn"
              >
                ➕ Add Server
              </button>
            ) : (
              <div className="add-server-form">
                <input
                  type="text"
                  value={newServerName}
                  onChange={(e) => setNewServerName(e.target.value)}
                  placeholder="Server name (e.g., 'ts1.travian.com')"
                  className="server-input"
                />
                <input
                  type="text"
                  value={newServerUrl}
                  onChange={(e) => setNewServerUrl(e.target.value)}
                  placeholder="SQL URL (e.g., 'https://ts1.travian.com/map.sql')"
                  className="server-input"
                />
                <div className="add-server-buttons">
                  <button 
                    onClick={addServer}
                    disabled={!newServerName.trim() || !newServerUrl.trim()}
                    className="save-server-btn"
                  >
                    💾 Save
                  </button>
                  <button 
                    onClick={() => {
                      setShowAddServer(false);
                      setNewServerName('');
                      setNewServerUrl('');
                    }}
                    className="cancel-server-btn"
                  >
                    ❌ Cancel
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>

        {loading ? (
          <div className="loading">Loading villages...</div>
        ) : (
          <div className="villages-grid">
            {villages.length > 0 ? (
              villages.map((village) => (
                <div key={village.id} className="village-card">
                  <h3>{village.name}</h3>
                  <div className="village-details">
                    <p><strong>Coordinates:</strong> ({village.x}, {village.y})</p>
                    <p><strong>Population:</strong> {village.population.toLocaleString()}</p>
                    {village.player && (
                      <p><strong>Player:</strong> {village.player}</p>
                    )}
                    {village.alliance && (
                      <p><strong>Alliance:</strong> {village.alliance}</p>
                    )}
                    {village.worldid && (
                      <p><strong>World ID:</strong> {village.worldid}</p>
                    )}
                  </div>
                </div>
              ))
            ) : (
              <div className="no-data">
                No villages found in this area
              </div>
            )}
          </div>
        )}
      </main>

      {showWorldInfo && worldInfo && (
        <WorldInfoModal 
          worldInfo={worldInfo} 
          onClose={() => setShowWorldInfo(false)} 
        />
      )}
    </div>
  )
}

export default App
