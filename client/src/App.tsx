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

function App() {
  const [villages, setVillages] = useState<Village[]>([]);
  const [serverStatus, setServerStatus] = useState<HealthResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string>('');
  const [isConnecting, setIsConnecting] = useState(false);
  const [sqlUrl, setSqlUrl] = useState<string>('');
  const [isLoadingSql, setIsLoadingSql] = useState(false);
  const [sqlMessage, setSqlMessage] = useState<string>('');
  
  const serverUrl = 'http://127.0.0.1:3001'; // Fixed server URL

  useEffect(() => {
    const fetchData = () => {
      setIsConnecting(true);
      setError('');
      
      Promise.all([fetchServerStatus(), fetchVillages()])
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

  const loadSqlFromUrl = async () => {
    if (!sqlUrl.trim()) {
      setSqlMessage('Please enter a SQL file URL');
      return;
    }

    setIsLoadingSql(true);
    setSqlMessage('');

    try {
      const response = await fetch(`${serverUrl}/api/load-sql`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ url: sqlUrl }),
      });

      if (response.ok) {
        const data = await response.json();
        setSqlMessage(data.message || 'SQL loaded successfully!');
        // Refresh the villages data
        await fetchVillages();
      } else {
        setSqlMessage('Failed to load SQL file');
      }
    } catch (err) {
      setSqlMessage('Failed to load SQL file');
      console.error('SQL load error:', err);
    } finally {
      setIsLoadingSql(false);
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

      <main>
        <div className="sql-config">
          <label htmlFor="sql-url">Load SQL Data:</label>
          <input
            id="sql-url"
            type="text"
            value={sqlUrl}
            onChange={(e) => setSqlUrl(e.target.value)}
            placeholder="Enter URL to SQL file (e.g., https://example.com/villages.sql)"
            className="sql-url-input"
          />
          <button 
            onClick={loadSqlFromUrl}
            disabled={isLoadingSql || !sqlUrl.trim()}
            className="load-sql-btn"
          >
            {isLoadingSql ? '🔄 Loading...' : '📥 Load SQL'}
          </button>
          {sqlMessage && (
            <div className={`sql-message ${sqlMessage.includes('Success') ? 'success' : 'error'}`}>
              {sqlMessage}
            </div>
          )}
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
