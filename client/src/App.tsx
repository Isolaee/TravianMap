import { useState, useEffect } from 'react'
import './App.css'

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

  const fetchMapData = async (x?: number, y?: number) => {
    try {
      // Use a subtle loading indicator instead of full loading state
      const url = x !== undefined && y !== undefined 
        ? `${serverUrl}/api/map?x=${x}&y=${y}`
        : `${serverUrl}/api/map`;
      
      const response = await fetch(url);
      if (response.ok) {
        const data = await response.json();
        setVillages(data);
        setError(''); // Clear error on success
      } else {
        setError('Failed to fetch map data');
      }
    } catch (err) {
      setError('Failed to fetch map data');
      console.error('Map fetch error:', err);
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

        <div className="controls">
          <button 
            onClick={() => fetchVillages()}
            disabled={isConnecting}
          >
            🏘️ Show All Villages
          </button>
          <button 
            onClick={() => fetchMapData(0, 0)}
            disabled={isConnecting}
          >
            📍 Show Near Origin (0,0)
          </button>
          <button 
            onClick={() => fetchMapData(10, 10)}
            disabled={isConnecting}
          >
            🗺️ Show Near (10,10)
          </button>
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
    </div>
  )
}

export default App
