# betweenworlds-tools
This repository contains tools to analyze the game between worlds https://betweenworlds.net/

All tools in this repository require api credentials, they include:
- authId - your account's username
- apiKey - AN api key that you can generate in the account settings

## Current tools
### A simple api library that interacts with the game's api
In the `betweenworlds-api` directory there is a partially complete library to interface with the game's api
The library requests and parses all responses.
Supported operation:
- get a user
- get all items
- get leaderboards
- get a user from a leaderboard
### A command line networth calculator
The networth calculator is located in the `networth` directory. You can use it by running `cargo run`.
note: Requires cargo.
To pass the credentials you have 2 options: 
1. Pass them as command line arguments `cargo run -- <authId> <apiKey>`
2. If you dont supply them in the command line arguments the program will ask you to input them via stdin.

### A player tracker
The player tracker is located in the `player-tracker` directory. You can use it by running `cargo run`
It can track players' leaderboard values such as credits and display it on a graph
#### Known bugs
- The graph bounds isnt calculated correctly when the graph goes in a straight line.
- The previous bug causes the graph to display incorrectly

#### Planned features
- Make the tracker offline so you don't need to keep the app open
- switch between themes
- Better error handling