# Travian Map - Empty Database Setup

## 🎯 Overview

You now have a **clean, empty PostgreSQL database** ready for real Travian server data. The old sample data has been completely removed.

## 📁 New Files Created

### 1. `init-empty.sql`
- **Purpose**: Initialize PostgreSQL with empty schema
- **Contains**: Basic servers table and utility functions
- **Used by**: Docker container initialization

### 2. `schema.sql` ⭐
- **Purpose**: Complete database schema creation and management
- **Contains**: Full schema with tables, indexes, functions, triggers, views
- **Usage**: Manual schema creation/recreation
- **Features**:
  - Complete database schema documentation
  - Utility functions for table management
  - Data validation and cleanup functions
  - Performance indexes
  - Sample data insertion (commented out)

### 3. `setup-empty-database.ps1`
- **Purpose**: Setup script for empty database
- **Features**: Clean setup without sample data

## 🗄️ Current Database State

✅ **PostgreSQL 15** running in Docker  
✅ **Empty database** - No sample data  
✅ **Servers table** created and ready  
✅ **Dynamic village tables** will be created as needed  
✅ **Rust application** connected and running  

## 🚀 How to Use

### Adding Your First Travian Server

1. **Through the API** (recommended):
```bash
curl -X POST http://127.0.0.1:3001/api/servers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Travian Server",
    "url": "https://ts1.travian.com"
  }'
```

2. **Directly in database**:
```sql
INSERT INTO servers (name, url, is_active) VALUES 
('Travian US1', 'https://ts1.travian.com', true);
```

3. **Set active server**:
```bash
curl -X POST http://127.0.0.1:3001/api/servers/1/activate
```

### Manual Schema Management

If you need to recreate or modify the schema:

```bash
# Connect to database
docker exec -it travian_map_postgres psql -U postgres -d travian_map

# Run the complete schema
\i /path/to/schema.sql

# Or from outside container
psql -h localhost -U postgres -d travian_map -f schema.sql
```

## 📊 Database Schema

### Core Tables
- **`servers`**: Manages multiple Travian servers
- **`villages_server_{id}_{date}`**: Dynamic tables for village data

### Key Features
- **Multi-server support**: Handle multiple Travian servers
- **Historical data**: Daily snapshots for growth tracking
- **Automatic cleanup**: Old data removal (keeps last 10 days)
- **Performance indexes**: Optimized for coordinate and population queries

## 🔧 Management Tools

### View Database
- **Adminer**: http://localhost:8080
- **Connection**: 
  - Server: `postgres`
  - Username: `postgres`
  - Password: `password`
  - Database: `travian_map`

### Useful Queries
```sql
-- List all servers
SELECT * FROM servers;

-- List all village tables
SELECT * FROM village_tables;

-- Count villages across all tables
SELECT * FROM count_all_villages();

-- Cleanup old tables (keeps last 10 days)
SELECT cleanup_old_village_tables(10);
```

## 🛠️ Development Notes

### Adding Real Data
1. Add your Travian server to the `servers` table
2. Set it as active (`is_active = true`)
3. The Rust application will automatically:
   - Fetch data from `{server_url}/map.sql`
   - Create daily tables
   - Parse and import village data
   - Generate statistics and analytics

### Data Flow
```
Travian Server → map.sql → Rust Parser → PostgreSQL → API → Frontend
```

## ✨ Next Steps

1. **Add your Travian servers** using the API or database
2. **Activate a server** to start data loading
3. **Start the frontend** to view your data:
   ```bash
   cd client
   npm install
   npm run dev
   ```
4. **Visit** http://localhost:5173 to see your Travian map!

---

🎉 **Your database is now ready for real Travian data!**
