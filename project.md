# Trusty-Vote: Discord STAR Voting Bot

## Project Overview
Trusty Vote (previously Rusty Bote) is a lightweight Discord bot written in Rust that allows server members to create and participate in polls using various voting methods—STAR voting, plurality, ranked choice, and approval. The bot provides an intuitive interface through slash commands and interactive components.

## Core Features
- Create polls via slash commands  
- Support for multiple voting methods:  
  - STAR (Score Then Automatic Runoff) voting  
  - Simple plurality voting  
  - Ranked choice voting  
  - Approval voting  
- Interactive voting through Discord buttons and select menus  
- Customizable poll duration (default: 24 hours, or manual close)  
- Automatic or manual poll closing  
- Clear results display  
- Lightweight and efficient design  

## Technical Architecture

### Technology Stack
- **Language**: Rust
- **Discord API**: [Serenity](https://github.com/serenity-rs/serenity)
- **Database**: PostgreSQL (with optional embedded Postgres for local development)
- **ORM**: [SQLx](https://github.com/launchbadge/sqlx)

### Data Persistence
The bot uses PostgreSQL for data storage. For local development, you can use the `embedded-postgres` feature to run a temporary Postgres instance without external setup. In production, set the `DATABASE_URL` environment variable to point to your Postgres server.

#### Database Schema
- **polls**: Stores poll metadata, including ID, question, voting method, timestamps, and status
- **poll_options**: Stores options for each poll, with position tracking
- **votes**: Records user votes with ratings for each poll option

### Discord Integration
- Utilizes Discord's slash commands API for command registration and handling
- Leverages Discord's message components (buttons, select menus) for interactive voting
- Uses embeds for visual presentation of polls and results
- **Required Permissions**: `View Channel`, `Send Messages`, `Embed Links`, `Read Message History`, `Manage Messages` (for updating poll messages). These should be requested during the bot invite or configured in server settings.

## User Flow

### Poll Creation
1. User invokes the `/poll` slash command
2. User provides:
   - Poll question
   - Options (candidates/choices)
   - Voting method selection
   - Poll duration (or manual close option)
3. Bot creates and posts the poll as an embed with interactive components

### Voting Process
1. Server members interact with buttons or select menus to cast votes
   - For STAR voting: Rate each option from 0-5 stars using select menus
   - For plurality voting: Select a single option
   - For ranked choice: Arrange options in order of preference
   - For approval voting: Toggle approval for any number of options
2. Votes are recorded in the database
3. Users can update their votes until the poll closes

### Results Calculation
1. When poll closes (automatically or manually):
   - Bot calculates results using the selected voting method
   - Results are displayed in an updated embed
   - For STAR voting: Shows both the scoring round and runoff round
   - For ranked choice: Shows elimination rounds

## Command Structure

### Primary Commands
- `/poll create` - Create a new poll  
- `/poll end [poll-id]` - Manually end an active poll  
- `/poll list` - Show active and recent polls in the server  
- `/poll help` - Display usage information and command help

### Help Subcommand Implementation
The `/poll help` subcommand provides a concise overview of Trusty-Vote. It summarizes the workflow, voting methods, and guides users through the bot's functionality.

### Poll Creation Parameters
- `question` - The poll question  
- `options` - The available choices (minimum: 2, maximum: 10)  
- `method` - Voting method (STAR, plurality, ranked choice, approval)  
- `duration` - Duration of the poll in minutes (default: 1440 = 24 hours, 0 = manual close)  

## Development Roadmap

### Current Status: Phase 2
- ✅ Core voting system implemented with all four voting methods
- ✅ Complete interaction handling architecture
- ✅ Database persistence for polls and votes
- ✅ Interactive UI components for all voting methods
- ✅ Poll lifecycle management (creation, voting, ending)
- ✅ Results calculation and display
- ✅ Comprehensive error handling and logging
- ✅ Poll listing functionality
- ✅ Help command with clear documentation

### Phase 2 Focus (Current)
- UI refinements and accessibility improvements
- Performance optimizations for larger servers
- Enhanced result visualizations
- Poll templates and configuration options

### Phase 3 Planning
- Integration with server roles for poll access control
- Scheduled polls
- Data export options
- Advanced analytics

## Voting Methods Implementation Details

### STAR Voting
Score Then Automatic Runoff:
1. **UI Implementation**: Interactive select menus that allow users to choose a 0-5 star rating for each option
2. **Data Structure**: Votes stored with option_id and rating values
3. **Results Calculation**: Two-phase process:
   - Scoring phase: Sum of ratings for each option
   - Runoff phase: Between the two highest-scoring options, the one preferred by more voters wins

### Plurality Voting
1. **UI Implementation**: Simple button interface with one click per option
2. **Data Structure**: One vote record per user with selected option
3. **Results Calculation**: Direct count of votes per option, highest total wins

### Ranked Choice Voting
1. **UI Implementation**: Interactive up/down/remove buttons to arrange preferences
2. **Data Structure**: Ordered array of option preferences per voter
3. **Results Calculation**: Elimination rounds with vote transfers until majority reached

### Approval Voting
1. **UI Implementation**: Toggle buttons for each option (approve/disapprove)
2. **Data Structure**: Array of approved options per voter
3. **Results Calculation**: Simple count of approvals per option, highest total wins

## Architecture Insights

### Component Interaction Flow
The Discord interaction system follows a structured pattern:
1. Incoming interaction received by `handle_interaction()`
2. Routed to appropriate handler based on type (command vs. component)
3. For components, the custom_id is parsed to determine:
   - Associated poll ID
   - Action type (vote button, star rating, approval toggle, etc.)
4. Poll status verification ensures closed polls reject new votes
5. Context-aware response generation based on interaction type and state

### Error Handling Strategy
The codebase implements a multi-layered error handling approach:
1. **User-facing errors**: Clear, actionable messages for permission issues or invalid inputs
2. **Comprehensive logging**: Error, warn, and info levels with context-rich messages
3. **Graceful degradation**: Component failures don't crash the entire application
4. **Context preservation**: Custom IDs and error contexts help debug issues
5. **Permission issues**: Detailed error messages that suggest permission requirements

## Technical Considerations

### Discord API Limitations
- **Component action rows**: Maximum 5 per message, limiting UI complexity
- **Interaction timeout**: 3-second response window requires efficient processing
- **Rate limits**: Managed with proper error handling and retry logic

### Database Optimizations
- **Indexed queries**: Poll retrieval optimized for active lookups
- **Transaction support**: Ensures vote integrity during concurrent operations
- **Query efficiency**: Minimized database round-trips in hot paths

### Scalability Considerations
- **Memory footprint**: Minimal state kept in memory between interactions
- **Connection pooling**: Efficient database connection management
- **Command registration**: Guild-specific vs. global command decisions

## Known Bugs and Fixed Issues
1. ✅ Fixed poll ID parsing for "done_voting_" buttons where the wrong array index was being used
2. ✅ Fixed ephemeral message handling - implemented proper follow-up responses
3. ✅ Added safety break for ranked choice algorithm to prevent infinite loops
4. ✅ Implemented truncation for result summaries exceeding Discord's embed field character limit (1024)
5. ✅ Improved error handling for missing permissions with actionable feedback

## Upcoming Enhancements
1. Advanced poll scheduling and time zone support
2. Role-based poll access control
3. Graphical representation of voting results using Unicode bar charts
4. Exportable results data

## Next Development Priorities

1. **Enhanced Results Visualization**
   - ✅ Add more detailed breakdowns of voting rounds to the results summary
   - Add graphical representations of voting outcomes (using Unicode blocks)
   - Support for exporting results data (e.g., CSV)

2. **Advanced Poll Configuration**
   - Role-restricted polls
   - Advanced scheduling options
   - Customizable voting thresholds

3. **Performance Optimizations**
   - Database query optimization for high-volume servers
   - Asynchronous processing improvements
   - Caching strategies for active polls

4. **UI/UX Improvements**
   - Modal dialogs for complex inputs
   - Improved mobile experience
   - Accessibility enhancements

## Implementation Notes
- All voting methods are fully implemented with working interfaces
- Database schema supports persistent storage across bot restarts (when using a real Postgres server; embedded Postgres is ephemeral by default)
- The command system has comprehensive subcommand support
- Advanced error handling with user feedback and detailed logging
- Discord component limitations are handled with appropriate UI design patterns
- For local development, use `cargo run --features embedded-postgres` to start the bot with an embedded Postgres instance (data is not persisted between runs). For production or persistent storage, set up a Postgres server and provide the connection string via the `DATABASE_URL` environment variable in your `.env` file.
