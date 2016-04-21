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
			Ok(Event::MessageCreate(message)) => {
				println!("{} says: {}", message.author.name, message.content);

				match message.content.as_ref() { 
					//Message format .send_message(channel, "Response", "Nonce", tts:bool);
					"!ping" => { 
					let pong = format!("{}, Pong", &message.author.name);
					let _ = discord.send_message(&message.channel_id, &pong , "", false);},
    				"!test" => { let _ = discord.send_message(&message.channel_id, "This is a reply to the test.", "", false); },
					"!help" => { let _ = discord.send_message(&message.channel_id, "If your seeking help from this bot you may not find it.", "", false); },
					"!Blah" => { let _ = discord.send_message(&message.channel_id, "Testing Nonce", "", false); },
					"!quit" => {println!("Quitting..."); break},
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


