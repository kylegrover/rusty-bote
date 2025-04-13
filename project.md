# Rusty-Bote: Discord STAR Voting Bot

## Project Overview
Rusty-Bote is a lightweight Discord bot written in Rust that allows server members to create and participate in polls using various voting methods—STAR voting, plurality, ranked choice, and approval. The bot provides an intuitive interface through slash commands and interactive components.

## Core Features
- Create polls via slash commands  
- Support for multiple voting methods:  
  - STAR (Score Then Automatic Runoff) voting  
  - Simple plurality voting  
  - Ranked choice voting  
  - Approval voting  
- Interactive voting through Discord buttons and select menus  
- Customizable poll duration  
- Automatic or manual poll closing  
- Clear results display  
- Lightweight and efficient design  

## Technical Architecture

### Technology Stack
- **Language**: Rust
- **Discord API**: [Serenity](https://github.com/serenity-rs/serenity)
- **Database**: SQLite for persistent storage
- **ORM**: [SQLx](https://github.com/launchbadge/sqlx)

### Data Persistence
The bot will use SQLite as its database solution, which provides:
- Lightweight footprint
- No separate database server required
- Good performance for the expected workload
- Simple backup and migration

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
   - Additional settings (e.g., anonymous voting)
3. Bot creates and posts the poll as an embed with interactive components

### Voting Process
1. Server members click on buttons to cast votes
   - For STAR voting: Rate each option from 0-5 stars using select menus
   - For other methods: Appropriate voting interface
2. Votes are recorded in the database
3. Users can update their votes until the poll closes

### Results Calculation
1. When poll closes (automatically or manually):
   - Bot calculates results using the selected voting method
   - Results are displayed in an updated embed
   - For STAR voting: Shows both the scoring round and runoff round

## Command Structure

### Primary Commands
- `/poll create` - Create a new poll  
- `/poll end [poll-id]` - Manually end an active poll  
- `/poll list` - Show active and recent polls in the server  
- `/poll help` - Display usage information and command help

### Help Subcommand Implementation
Now, the `/poll help` subcommand provides a short, docs-like overview of Rusty-Bote. It summarizes the usual workflow, voting methods, and points users to the documentation for advanced features.

### Poll Creation Parameters
- `question` - The poll question  
- `options` - The available choices (minimum: 2, maximum: 10)  
- `method` - Voting method (STAR, plurality, ranked choice, approval)  
- `duration` - Duration of the poll in minutes (0 = manual close)  
- `anonymous` - Tracks whether votes should be anonymous (planned, not fully implemented yet)

## Development Roadmap

### Current Status: Late Phase 1 / Early Phase 2
- ✅ Core voting system implemented with all four voting methods
- ✅ Complete interaction handling architecture
- ✅ Database persistence for polls and votes
- ✅ Interactive UI components for all voting methods
- ✅ Poll lifecycle management (creation, voting, ending)
- ✅ Results calculation and display

### Phase 2 Focus (Current)
- Comprehensive error handling and user feedback
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

## Known Bugs and Ongoing Fixes
1. Fixed an incomplete function in the database module that was missing its brackets and signature, causing build issues.
2. Confirmed ephemeral messages do not return a true message ID, so ephemeral poll messages cannot be edited or tracked. This is expected behavior but worth noting for future UI changes.
3. Ranked Choice calculation could potentially enter an infinite loop in edge cases (added safety break). Needs further testing with complex tie scenarios.
4. Embed field value limit (1024 chars) might truncate very detailed results summaries. Implemented basic truncation.

## Upcoming Enhancements
1. Advanced scheduling for polls (Phase 3).
2. Better ephemeral poll handling or alternative UI for ephemeral interactions.

## Next Development Priorities

1. **Enhanced Results Visualization**
    *   ✅ Add more detailed breakdowns of voting rounds to the results summary (STAR scoring/runoff, Ranked Choice rounds).
    *   Add graphical representations of voting outcomes (e.g., bar charts using Unicode blocks or image generation).
    *   Support for exporting results data (e.g., CSV).

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
- Database structures support persistent storage across bot restarts
- The command system supports all planned subcommands
- Comprehensive error handling with user feedback is in place
- Discord component limits are handled gracefully with appropriate UI designs

## Scratch Pad 
The LLM assistant can use the area below to keep notes about the state of the project, such as tracking to-dos, notes about the details of a current task or how an error is impacting it, or save important snippets from the users prompts such as provided docs. This are is impermanent so the LLM is free to delete anything below this line as it's working if it thinks it's no longer relevant.
---

### Current Progress
- ✅ Basic project structure set up
- ✅ Discord bot framework with Serenity
- ✅ Command registration system for `/poll` commands
- ✅ SQLite database connection and schema created
- ✅ Poll creation with interactive embed
- ✅ Fixed ownership issues in STAR voting module
- ✅ Added Display trait implementation for VotingMethod
- ✅ Implemented proper poll ending with results display
- ✅ Fixed STAR voting interface with proper button components
- ✅ Improved STAR voting UI with clear option labels
- ✅ Removed unnecessary vote acknowledgment messages
- ✅ Implemented Ranked Choice voting algorithm
- ✅ Added clear labels for options in voting interface
- ✅ Hide "Cast Your Vote" button for closed polls
- ✅ Implemented all voting method interfaces
- ✅ Added result calculation for all voting methods
- ✅ Refactored component handler to correctly parse Poll ID and check status
- ✅ Fixed compiler errors related to handler signatures
- ✅ Implemented poll listing functionality
- ✅ Added help command with usage instructions
- ✅ Enhanced error handling with user-friendly messages
- ✅ Implemented comprehensive logging system
- ✅ Added permission checks and informative error messages
- ✅ Improved results summary detail for STAR and Ranked Choice voting.

### Next Tasks
1. Implement results visualization enhancements
   - ✅ Provide more detailed breakdowns of runoff rounds/scoring.
   - Add charts/graphs for vote distribution (text-based or image).
   - Implement results export functionality.

2. Optimize database queries
   - Add indices for common lookup patterns
   - Implement caching for frequently accessed polls
   - Consider connection pooling improvements

3. Enhance UI for complex polls
   - Explore modal dialogs for polls with many options
   - Design alternative interfaces for polls exceeding component limits
   - Improve mobile responsiveness

4. Add administrative features
   - Poll management dashboard for server admins
   - Ability to clone/template previous polls
   - Advanced scheduling options
