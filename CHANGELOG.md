# Changelog

All notable changes to Battle of Culiacán RTS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-01-30

### Added - Phase 6A: Enhanced Political Dynamics & Multiplayer Foundation
- **Political Pressure System**: Comprehensive government decision-making mechanics with stability tracking
  - Government Stability: Dynamic political will calculation based on casualties, media attention, and operation duration
  - Public Opinion System: Social media influence affects government and cartel support levels
  - Political Decision Thresholds: Realistic government capitulation mechanics leading to historical outcome
  - Active Politicians: President, Defense Minister, Interior Minister, State Governor with individual pressure levels
- **Social Media Influence System**: Real-time viral content generation and hashtag trending
  - Viral Content Generation: Combat footage, civilian displacement, property damage content with sentiment analysis
  - Hashtag Trends: Dynamic trending topics (#Culiacan, #OvidioGuzman, #EndTheViolence) with engagement metrics
  - International Coverage: Media attention affects international pressure on government
  - Twitter Sentiment & Facebook Engagement: Social media metrics influencing public opinion
- **Multiplayer Networking Foundation**: Complete infrastructure for 4 game modes
  - Asymmetric Mode: Government vs Cartel (2-8 players) with role-based gameplay
  - Historical Mode: Recreate exact Oct 17, 2019 scenario with historical constraints
  - Cooperative Mode: Team-based survival against AI with shared objectives
  - Competitive Mode: Multiple factions competing for territorial control
  - Authentication Integration: Player sessions using existing OAuth system (Google, GitHub, Discord)
  - Network Message System: Real-time synchronization of game state across clients
- **Real-time Political UI**: Live display of political status with color-coded indicators
  - Government Stability Meter: Visual representation of political will (Green/Yellow/Red)
  - Media Attention Tracker: Shows international coverage and pressure levels
  - Casualty Counter: Real-time civilian and military casualty tracking
  - Operation Duration: Live timer showing elapsed operation time
  - Recent Events Feed: Latest political developments and government responses

### Enhanced
- **Campaign System**: Integrated with political pressure mechanics affecting mission outcomes
- **Government Decision System**: AI government responds realistically to political pressure
- **Authentication System**: Extended with multiplayer session management
- **UI System**: Added political status panels with live updates

### Technical
- **Political System Plugin**: New modular system with 5 integrated sub-systems
- **Multiplayer System Plugin**: Network infrastructure with session management
- **Social Media Resources**: Hashtag trends, viral content tracking, engagement metrics
- **Network Manager**: Client-server architecture with authentication integration
- **Political UI Components**: Real-time status panels with dynamic color coding

## [0.2.0] - 2025-01-30

### Added - Phase 4C: Environmental Systems
- **Dynamic Weather System**: 5 weather types (Clear, Overcast, LightRain, HeavyRain, Fog) with random transitions every 2-5 minutes
- **Time-of-Day Progression**: Historical accuracy starting at late afternoon (Oct 17, 2019) with dynamic lighting
- **Environmental Gameplay Effects**: Weather impacts unit movement speed and detection range
  - Heavy Rain: 40% visibility reduction, 20% movement penalty
  - Dense Fog: 60% visibility reduction - favors close-quarters combat
  - Light Rain: 20% visibility reduction, 5% movement penalty
  - Overcast: 10% visibility reduction
  - Clear: Full unit effectiveness
- **Visual Weather Effects**: Rain droplets and fog cloud particles with realistic physics
- **Dynamic Ambient Lighting**: Color and intensity changes based on time and weather conditions
- **Console Environmental Feedback**: Real-time status updates and tactical implications
- **Movement System Integration**: Environmental modifiers affect unit movement speed
- **Combat Detection Integration**: Visibility modifiers affect unit detection ranges

### Enhanced
- **Combat System**: Now uses environmental visibility modifiers for detection ranges
- **Movement System**: Now applies weather-based movement penalties
- **Tactical Feedback**: Added environmental status displays every 30 seconds
- **Weather Transitions**: Enhanced with tactical impact descriptions

### Technical
- **Environmental Resources**: Added `EnvironmentalState` and `EnvironmentalAmbientLight` resources
- **Weather Particles**: New particle system for rain and fog effects
- **Time-of-Day Calculations**: Proper day/night cycle with lighting transitions
- **System Integration**: Environmental systems fully integrated with movement and combat

## Previous Releases

### Phase 5E: Technical Enhancements ✅ **COMPLETED**
- Modular Architecture: 15+ specialized modules replacing monolithic code
- Save System: 10 save slots with campaign progress and metadata
- Configuration System: Comprehensive settings with JSON persistence
- Performance Monitoring: FPS tracking, frame time analysis, diagnostics
- Hotkey Support: F11 fullscreen, F3 FPS display, Ctrl+S save config

### Phase 5D: Campaign Structure ✅ **COMPLETED**
- 13 Historical Missions: Complete Oct 17, 2019 timeline (3:15 PM - 8:30 PM)
- Neighborhood Maps: Las Flores, Tierra Blanca, Centro, Las Quintas, Airport
- Political Pressure System: Dynamic mechanics affecting government decisions
- Progressive Difficulty: Historically accurate escalation and resolution
- Mission Phases: InitialRaid → UrbanWarfare → PoliticalNegotiation → Resolution

### Phase 5C: Enhanced Gameplay ✅ **COMPLETED**
- Camera System: Pan, zoom, follow units with smooth transitions
- Unit Selection: Multi-select with formation controls
- Advanced AI: Tactical AI with squad coordination
- Environmental Gameplay: Weather affects visibility and movement
- Special Abilities: Unique cartel and military capabilities

### Phase 5B: Advanced Audio ✅ **COMPLETED**
- Spatial Audio System: 3D positioned sound effects with distance attenuation
- Professional Audio Assets: 30+ .ogg files (combat, ambient, music, radio, UI)
- Dynamic Music System: Background tracks with smooth transitions
- Radio Chatter: Faction-specific communications with historical authenticity
- Console Audio Fallback: Dual pipeline for maximum compatibility

### Phase 5A: Asset Integration ✅ **COMPLETED**
- Pixel Art Sprites: Custom 48x48 unit sprites integrated
- Modern Exteriors Tileset: Professional 16x16 city tileset with 140+ assets
- Animation System: Basic unit movement and combat animations
- Asset Pipeline: Organized asset structure with proper categorization