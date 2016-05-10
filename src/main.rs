extern crate discord;
extern crate hyper;
extern crate serde_json;

use discord::{Discord, State};
use discord::model::Event;
use std::env;
use std::error::Error;

fn get_cat() -> Result<String, Box<Error>> {
    let url: &str = "http://random.cat/meow";
    use std::io::Read;
    use serde_json::Value;

    let client = hyper::Client::new();
    let mut response = try!(client.get(url).send());
    let mut buff = String::new();
    try!(response.read_to_string(&mut buff));
    let decode: Value = try!(serde_json::from_str(&buff));
    let data = decode.as_object().expect("Invalid JSON");
    let file = data.get("file").expect("File not found");
    Ok(file.as_string().unwrap().to_string())
}

fn main() {
    let discord = Discord::from_bot_token(&env::var("RUST_BOT_TOKEN").expect("DISCORD TOKEN")).expect("Discord Token Error");
	let (mut connection, ready) = discord.connect().expect("Connection Failed.");
	println!("Rust Bot Ready...");

    let mut state = State::new(ready);

    loop {
        let event = match connection.recv_event() {
            Ok(event) => event,
            Err(err) => {
                println!("[Warning] Receive error: {:?}", err);
				if let discord::Error::WebSocket(..) = err {
					// Handle the websocket connection being dropped
					let (new_connection, ready) = discord.connect().expect("connect failed");
					connection = new_connection;
					state = State::new(ready);
					println!("[Ready] Reconnected successfully.");
				}
				if let discord::Error::Closed(..) = err {
					break
				}
				continue
            },
        };

        state.update(&event);

        match event {
			Event::MessageCreate(message) => {
                use std::ascii::AsciiExt;
				println!("{} says: {}", message.author.name, message.content);

                let mut split = message.content.split(" ");
                let command = split.next().unwrap_or("");
                let argument = split.next().unwrap_or("");

                let voice_channel = state.find_voice_user(message.author.id);

                if command.eq_ignore_ascii_case("/play") {
                    println!("Play command called");

                    if let Some((server_id, channel_id)) = voice_channel {
                        let voice = connection.voice(server_id);
                        voice.connect(channel_id);
                        if !argument.eq_ignore_ascii_case(""){
                            voice.play(discord::voice::open_ytdl_stream(&argument).unwrap());
                        }
                    } else {
                        let _ = discord.send_message(&message.channel_id, "You must be in a voice channel to play Music", "", false);
                    }
                }
                if command.eq_ignore_ascii_case("/end"){
                    if let Some((server_id, _)) = voice_channel {
                        connection.drop_voice(server_id);
                    }
                }
                match message.content.as_ref() {

                    "/cat" => {
                        if let Ok(s) = get_cat() {
                            println!("{}", s);
                            let _ = discord.send_message(&message.channel_id, &s, "", false);
                        }
                    },
                    "/ping" => {
                        let pong = format!("<@{:?}>, Pong", &message.author.id.0);
                        let _ = discord.send_message(&message.channel_id, &pong , "", false);
                    },
                    "/info" => { let _ = discord.send_message(&message.channel_id,
                        "Rust bot was programmed in Rust Lang, using Discord-rs: https://github.com/SpaceManiac/discord-rs.", "", false);
                    },
                    "/test" => { let _ = discord.send_message(&message.channel_id,
                        "This is a reply to the test.", "", false);
                    },
                    "/help" => { let _ = discord.send_message(&message.channel_id,
                        "If your seeking help from this bot you may not find it.", "", false);
                    },
                    "/quit" => {println!("Quitting..."); break},
                        _ => continue,
                }
			}
            Event::VoiceStateUpdate(server_id, _) => {
				// If someone moves/hangs up, and we are in a voice channel,
				if let Some(cur_channel) = connection.voice(server_id).current_channel() {
					// and our current voice channel is empty, disconnect from voice
					if let Some(srv) = state.servers().iter().find(|srv| srv.id == server_id) {
						if srv.voice_states.iter().filter(|vs| vs.channel_id == Some(cur_channel)).count() <= 1 {
							connection.voice(server_id).disconnect();
						}
					}
				}
			}
			_ => {}
		}
	}
	// Log out from the API
	discord.logout().expect("logout failed");
}
