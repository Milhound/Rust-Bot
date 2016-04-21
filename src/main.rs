extern crate discord;

use discord::Discord;
use discord::model::Event;

use std::env;

fn main() {
	// Log in to Discord using the email and password in the environment
	let discord = Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN")).expect("Discord Token Error");

	// Establish and use a websocket connection
	let (mut connection, _) = discord.connect().expect("Connection failed.");
	println!("Ready...");
	loop {
		match connection.recv_event() {
		/*
			Ok(Event::MessageCreate(message)) => {
				pub struct Message {
			    pub id: MessageId,
			    pub channel_id: ChannelId,
			    pub content: String,
			    pub nonce: Option<String>,
			    pub tts: bool,
			    pub timestamp: String,
			    pub edited_timestamp: Option<String>,
			    pub author: User,
			    pub mention_everyone: bool,
			    pub mentions: Vec<User>,
			    pub attachments: Vec<Attachment>,
			    pub embeds: Vec<Value>,
			}
		*/
			
				println!("{} says: {}", message.author.name, message.content);

				match message.content.as_ref() { 
					//fn send_message(&self, channel: &ChannelId, text: &str, nonce: &str, tts: bool) -> Result<Message>
					"!ping" => { 
					// Format! used to convert (String::collections, &str) to simple &str
					//{:?} used to unwrap tuple over {} for literal
					//USER_ID = pub struct UserId(pub u64);
					//Use .0 to access the first value of a struct like an Array
					
					// Pong returns @user, Pong
					let pong = format!("<@{:?}>, Pong", &message.author.id.0);
					let _ = discord.send_message(&message.channel_id, &pong , "", false);},
					// Info returns Rust bot was programmed in Rust Lang, using Discord-rs: https://github.com/SpaceManiac/discord-rs.
					"/info" => { let _ = discord.send_message(&message.channel_id, "Rust bot was programmed in Rust Lang, using Discord-rs: https://github.com/SpaceManiac/discord-rs.", "", false);},
    				// Test returns This is a reply to the test.
    				"/test" => { let _ = discord.send_message(&message.channel_id, "This is a reply to the test.", "", false); },
					// Help returns If your seeking help from this bot you may not find it.
					"/help" => { let _ = discord.send_message(&message.channel_id, "If your seeking help from this bot you may not find it.", "", false); },
					// Force Closes the bot
					"/quit" => {println!("Quitting..."); break},
					_ => continue,
					
				}

			}
			Ok(_) => {}

			Err(err) => println!("Receive error: {:?}", err)
		}
	}

	// Log out from the API
	discord.logout().expect("logout failed");
}


