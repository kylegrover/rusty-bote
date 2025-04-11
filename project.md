# Rusty-Bote: Discord STAR Voting Bot

## Project Overview
Rusty-Bote is a lightweight Discord bot written in Rust that allows server members to create and participate in polls using various voting methodologies, with STAR voting as the primary method. The bot provides an intuitive interface through Discord's slash commands and interactive components.

## Core Features
- Create polls via slash commands
- Support for multiple voting methods:
  - STAR (Score Then Automatic Runoff) voting
  - Simple plurality voting
  - Ranked choice voting
  - Approval voting
- Interactive voting through Discord buttons
- Customizable poll duration
- Automatic or manual poll closing
- Clear results visualization
- Lightweight and efficient design

## Technical Architecture

### Technology Stack
- **Language**: Rust
- **Discord API**: [Serenity](https://github.com/serenity-rs/serenity) or [Twilight](https://github.com/twilight-rs/twilight)
- **Database**: SQLite for persistent storage
- **ORM**: [SQLx](https://github.com/launchbadge/sqlx) or [Diesel](https://diesel.rs/)

### Data Persistence
The bot will use SQLite as its database solution, which provides:
- Lightweight footprint
- No separate database server required
- Good performance for the expected workload
- Simple backup and migration

### Discord Integration
- Utilizes Discord's slash commands API for command registration and handling
- Leverages Discord's message components (buttons) for interactive voting
- Uses embeds for visual presentation of polls and results

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
   - For STAR voting: Rate each option from 0-5 stars
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
- `/poll list` - Show active polls in the server
- `/poll help` - Display help information about voting methods

### Poll Creation Parameters
- `question` - The poll question
- `options` - The available choices
- `method` - Voting method (STAR, plurality, ranked choice, approval)
- `duration` - How long the poll should run
- `anonymous` - Whether votes should be publicly visible

## Development Roadmap

### Phase 1: MVP
- Basic STAR voting implementation
- Poll creation and voting interface
- Simple results display
- SQLite persistence

### Phase 2: Enhanced Features
- Additional voting methods
- Improved result visualization
- Poll templates and saving
- Admin controls

### Phase 3: Advanced Features
- Integration with server roles for poll access control
- Scheduled polls
- Data export options
- Advanced analytics

## Voting Methods

### STAR Voting
Score Then Automatic Runoff:
1. Voters rate each candidate from 0-5 stars
2. The two candidates with the highest total scores advance to a runoff
3. In the runoff, the candidate preferred by more voters wins

### Other Methods
- **Plurality**: Traditional "most votes wins" approach
- **Ranked Choice**: Voters rank candidates; elimination rounds until majority
- **Approval**: Voters approve or disapprove each option; most approvals wins

## Technical Considerations
- Efficient handling of Discord API rate limits
- Proper database indexing for quick poll retrieval
- Secure handling of vote data
- Scalable architecture for use across many Discord servers

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

### Next Tasks
1. ✅ Implement interactive voting buttons for STAR voting
   - ✅ Create message components for rating options 0-5
   - ✅ Handle component interactions to record votes
2. ✅ Implement poll ending and result calculation
   - ✅ Fetch and calculate STAR voting results
   - ✅ Display results visually in an embed
3. Add support for other voting methods
   - Implement Plurality voting calculation
   - Implement Ranked Choice voting calculation
   - Implement Approval voting calculation
4. Implement `/poll list` command
5. Implement `/poll help` command
