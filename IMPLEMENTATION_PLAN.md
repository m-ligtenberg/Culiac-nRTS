# Implementatieplan voor verbetering van modules

Dit plan beschrijft de verbeteringen die we in de huidige modules gaan doorvoeren. Het doel is de code leesbaarder, robuuster en consistenter te maken. Hieronder per module worden de belangrijkste verbeterpunten opgesomd:

## 1. src/audio_system.rs
- **Aanpassing logging:** Verander alle `println!`-aanroepen (voor debug/info fallback) naar gebruik van logging macros zoals `info!` (of `warn!`/`error!` indien van toepassing).
- **Debugginstrumenten:** Voeg extra logging toe bij het ontbreken van audio-bestanden.
- **Spatial audio optimalisatie:** Zorg dat de logica rondom 3D-audio consistent is en documenteer de werking van de spatial audio.

## 2. src/auth/errors.rs
- **Logging en foutafhandeling:** Verfijn de foutboodschappen waar nodig en controleer of alle fouttypen correct worden afgehandeld.
- **Documentatie:** Voeg inline commentaar toe om de mapping tussen fouttypen en HTTP-statuscodes te verduidelijken.

## 3. src/auth/db_init.rs
- **Logging:** Vervang alle `println!`-aanroepen door `info!`, `warn!` of `error!` om consistentie in de logging te waarborgen.
- **Foutafhandeling:** Zorg dat migraties en opschoning van verlopen data robuust en traceerbaar worden afgehandeld.

## 4. src/auth/jwt.rs
- **Extra validatie en logging:** Voeg extra logging toe bij token generatie en validatie, zodat bij fouten snel inzicht wordt verkregen.
- **Error mapping:** Controleer de mapping van JWT-fouten zodat deze in lijn is met de algehele foutafhandeling van de applicatie.

## 5. src/config.rs
- **Logging:** Vervang `println!` bij reset en hotkey-acties door consistente logging (bijv. `info!`).
- **Validatie:** Controleer of de validatiefunctie voldoende waarschuwingen geeft en pas logging hierop toe.

## 6. src/multiplayer_system.rs
- **Robuustheid:** Zorg dat alle netwerkintegratie en synchronisatiesystemen robuust omgaan met verbindingsverliezen en timeouts.
- **Role & Assignment Logging:** Documenteer en log belangrijke gebeurtenissen bij het toewijzen van rollen en start van multiplayer games.

## 7. src/save_system.rs
- **Foutafhandeling en logging:** Voeg gedetailleerde logging toe bij opslaan, laden en verwijderen van savefiles.
- **Bestandsbeheer:** Controleer of directory’s correct worden aangemaakt en valideer de ‘slot’ nummers duidelijk.

## 8. src/ui/ui_core.rs
- **UI Optimalisaties:** Controleer of de UI-update logica goed gescheiden en efficiënt is. Voeg inline commentaar toe voor toekomstige onderhoud.
- **Consistente tekstformattering:** Zorg dat alle tekstweergaven (status, score, wave, difficulty) consistent worden bijgewerkt en gestileerd.

## 9. src/utils/spatial.rs
- **Optimalisatie Spatial Grid:** Verbeter de implementatie van de spatial partitioning door het gebruik van een grid. Zorg voor een preciezere berekening van nabije cellen en buurcellen.
- **Documentatie:** Voeg uitleg toe over de werking van de grid-cellen en de zoekmethode voor nabijheid.

## 10. src/utils/unit_queries.rs
- **Algoritme herschrijving:** Zorg voor performante filtering van eenheden (als ally/enemy) met eventueel hergebruik van bestaande helper functies.
- **Consistency check:** Controleer dat de methodes count_nearby_allies en count_nearby_enemies optimaal werken en zorg dat er inline documentatie bij staat.

## 11. src/auth/database.rs
- **Query optimalisatie en error logging:** Verbeter de error handling en logging bij database-operaties.
- **Veiliger maken migraties:** Zorg dat alle CRUD-operaties met gebruikers omtrent veiligheidsaspecten (zoals hashing en validaties) worden nageleefd.

---

**Algemene aanpak:**
1. **Logging Consistentie:** Zorg dat overal de `info!`, `warn!` en `error!` logging macros gebruikt worden, in plaats van `println!`.
2. **Robuuste Error Afhandeling:** Zorg dat alle modules gebruik maken van uniforme fouttypes en -afhandeling.
3. **Code Documentatie en Commentaar:** Voeg uitleg en commentaar toe in kritieke functies.
4. **Unit Tests Uitbreiden:** Voeg tests toe waar nodig om de functionaliteit van verbeterde functies te garanderen.

Implementatie dient gefaseerd te gebeuren, module per module, zodat eventuele regressies eenvoudig kunnen worden geïdentificeerd via de bestaande test suites.
