extern crate discord;
extern crate hyper;
extern crate serde_json;

use discord::Discord;
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
    println!("{:?}", &env::var("RUST_BOT_TOKEN").expect("DISCORD TOKEN"));
	let (mut connection, _) = discord.connect().expect("Connection Failed.");
	println!("Ready...");

    loop {
		match connection.recv_event() {
			Ok(Event::MessageCreate(message)) => {
				println!("{} says: {}", message.author.name, message.content);
                match message.content.as_ref() {

                    "/cat" => {
                        if let Ok(s) = get_cat() {
                            println!("{}", s);
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
			Ok(_) => {}
			Err(discord::Error::Closed(code, body)) => {
				println!("Gateway closed on us with code {:?}: {}", code, String::from_utf8_lossy(&body));
				break
			}
			Err(err) => println!("Receive error: {:?}", err)
		}
	}
	// Log out from the API
	discord.logout().expect("logout failed");
}
