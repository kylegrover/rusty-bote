# 🛥️ Rusty Bote

toot toot

*Lightweight rust discord bot for running alternative polling/voting methods such as STAR*

see **project.md** for project info. project.md is also designed to be passed to an LLM along with any prompts for assistance with the project.

⚠️ this thing is made of pure vibes ⚠️

### links and info

discord app config: https://discord.com/developers/applications/1360286244856402222/installation
bot install link: https://discord.com/oauth2/authorize?client_id=1360286244856402222

bot pfp is from: https://pixabay.com/photos/mediterranean-sea-old-ship-rusty-112004/

### running locally

1. set up your own discord app, get the token and add it to .env, invite the bot to your server

2a. `cargo run --features embedded-postgres`

2b. or set up a local postgres server and provide it via DATABASE_URL .env var

the bot should add its slash commands to your server and you can interface with it as normal. the embedded-postgres provides an sqlite experience where you can run the bot in one file, but it is currently not persisted between runs. i think that's possible and may be added in the future