# Project Todos

## Active
- [ ] Fix module imports in main.rs (audio_system, multiplayer_system references)
- [ ] Fix import path in campaign.rs (save_system -> save::save_system)
- [ ] Fix import path in ui_menus.rs (save_system -> save::save_system)
- [ ] Create missing utility functions in utils module
- [ ] Fix borrow checker error in multiplayer_system.rs line 388
- [ ] Clean up 61 debug statements (println!) throughout codebase
- [ ] Fix 73 compiler warnings (unused variables, imports, etc.)
- [ ] Run cargo fmt to fix formatting inconsistencies
- [ ] Implement complete spatial grid benchmarking system
- [ ] Add comprehensive unit tests for core game systems
- [ ] Implement missing AI coordination features
- [ ] Complete multiplayer authentication integration
- [ ] Add error handling for save/load operations
- [ ] Implement campaign progression validation
- [ ] Add asset loading error handling
- [ ] Optimize performance bottlenecks in game systems
- [ ] Add comprehensive documentation for public APIs
- [ ] Implement proper logging configuration
- [ ] Add configuration validation and defaults
- [ ] Implement proper resource cleanup on game exit
- [ ] **Phase 5F: Enhanced Cartel Faction "Cool Factor"** - Implement mechanically engaging cartel gameplay without glorifying violence: 1) Mechanical improvements (unique abilities like smoke grenades, rapid roadblock deployment, street-smart tactics, underdog mobility/agility mechanics), 2) Aesthetic enhancements (distinctive unit designs, gritty UI themes, intense suspenseful audio, confident battle chatter), 3) Player experience upgrades (high difficulty requiring skill/quick thinking, advanced strategy discovery, replay value through branching missions, morale system, urban camouflage mechanics, resourceful upgrades). Focus on player skill and distinct identity while maintaining historical objectivity.

## Completed
- [x] Created missing module files (audio_system.rs, multiplayer_system.rs, save_system.rs) | Done: 07/31/2025
- [x] Fixed duplicate function definitions in ai.rs | Done: 07/31/2025
- [x] Added missing log imports for auth modules | Done: 07/31/2025
- [x] Created proper module structure with mod.rs files | Done: 07/31/2025
- [x] **Phase 5D: Campaign Structure** ✅ **COMPLETED** - 13 historical missions covering complete Oct 17, 2019 timeline with political pressure mechanics
- [x] **Phase 5E: Technical Enhancements** ✅ **COMPLETED** - Modular architecture, save system, configuration, and performance monitoring
- [x] **Phase 5A: Asset Integration** ✅ **COMPLETED** - Professional sprites and audio assets
- [x] **Phase 5B: Advanced Audio** ✅ **COMPLETED** - Spatial audio system with 30+ .ogg files
- [x] **Phase 5C: Enhanced Gameplay** ✅ **COMPLETED** - Camera controls, unit selection, advanced AI
