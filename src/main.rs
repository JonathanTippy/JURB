mod util;

extern crate discord;

use discord::model::{Event, Reaction, ReactionEmoji, ChannelId, ServerId, MessageId};
use discord::Discord;
use discord::Connection;
use std::env;
use std::time::Instant;
use std::vec;
use util::{get_rating, cull_meme_cache, reproduce_message};

use std::collections::HashMap;

use simple_user_input::get_input;

mod simple_user_input {
    use std::io;
    pub fn get_input(prompt: &str) -> String{
        println!("{}",prompt);
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_goes_into_input_above) => {},
            Err(_no_updates_is_fine) => {},
        }
        input.trim().to_string()
    }
}

const BOT_CACHE_NAME:&str = "botcache";

const MEME_CACHE_LINES:usize = 30; // more than about thirty might hit the character limit

struct ServerMap {
    memes_channel_id: u64
    , funny_memes_channel_id: u64
}

fn main() {


    let mut channel_to_server_map = HashMap::new();
    let mut funny_channels_map = HashMap::new();
    let mut original_to_reproduction_map = HashMap::new();
    let mut serverid_to_cache_map = HashMap::new();

    // Log in to Discord using a bot token from the environment
    let start = Instant::now();
    let discord = Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("Expected token"))
        .expect("login failed");

    // Establish and use a websocket connection
    let (mut connection, ready_event) = discord.connect().expect("connect failed");
    let duration1 = start.elapsed();

    let start = Instant::now();

    let servers = discord.get_servers().expect("Failed to fetch servers");


    //let mut botcache_id = MessageId(0);
    //let mut botcache_channel_id = ChannelId(0);
    for server in servers {
        let channels = discord.get_server_channels(server.id).unwrap();
        let mut found:bool = false;
        for channel in channels {
            channel_to_server_map.insert(channel.id, server.id);
            if channel.name == BOT_CACHE_NAME {

                match channel.last_message_id {
                    Some(message_id) => {
                        match discord.get_message(channel.id, message_id) {
                            Ok(message) => {let map = message.content.split("\n").collect::<Vec<_>>()[0];
                                // retrieve channels map
                                funny_channels_map.insert(
                                    ChannelId(map.split(":").collect::<Vec<_>>()[0].parse().unwrap())
                                    , ChannelId(map.split(":").collect::<Vec<_>>()[1].parse().unwrap())
                                );
                                // retrieve meme maps
                                for line in message.content.split("\n") {
                                    if line==map {continue}
                                    original_to_reproduction_map.insert(
                                        MessageId(line.split(":").collect::<Vec<_>>()[0].parse().unwrap())
                                        , MessageId(line.split(":").collect::<Vec<_>>()[1].parse().unwrap())
                                    );
                                }
                                // remember cache
                                serverid_to_cache_map.insert(server.id, (message.id, channel.id));
                                // cull cache
                                cull_meme_cache(&discord, &message, MEME_CACHE_LINES);
                            }

                            Err(message) => {
                                // get channels map from user
                                let map: String = get_input("please provide the 'memes' to 'funny memes' map in the form of ID:ID");
                                // create message & cache channels map
                                let sent = discord.send_message(channel.id, &map, "", false).unwrap();
                                funny_channels_map.insert(
                                    ChannelId(map.split(":").collect::<Vec<_>>()[0].parse().unwrap())
                                    , ChannelId(map.split(":").collect::<Vec<_>>()[1].parse().unwrap())
                                );
                                // remember cache
                                serverid_to_cache_map.insert(server.id, (sent.id, channel.id));
                            }
                        }
                    }
                    None => {
                        // get map from user
                        let map: String = get_input("please provide the 'memes' to 'funny memes' map in the form of ID:ID");
                        // save map
                        let sent = discord.send_message(channel.id, &map, "", false).unwrap();
                        funny_channels_map.insert(
                            ChannelId(map.split(":").collect::<Vec<_>>()[0].parse().unwrap())
                            , ChannelId(map.split(":").collect::<Vec<_>>()[1].parse().unwrap())
                        );
                        // remember cache
                        serverid_to_cache_map.insert(server.id, (sent.id, channel.id));
                    }
                }
                found = true;
            }
        }
    }

    let duration2 = start.elapsed();


    println!("Time elapsed connecting: {:?}", duration1);
    println!("Time elapsed finding caches: {:?}", duration2);
    println!("Ready.");
    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                println!("{} says: {}", message.author.name, message.content);
                if message.content == "!test" {
                    let start = Instant::now();
                    let _ = discord.send_message(
                        message.channel_id,
                        "This is a reply to the test.",
                        "",
                        false,
                    );
                    let duration = start.elapsed();
                    println!("Time elapsed sending a message: {:?}", duration);
                } else if message.content.contains("s this true") && message.content.contains("@") {
                    let start = Instant::now();
                    if rand::random() {
                        let _ = discord.send_message(
                            message.channel_id,
                            "yup, its true.",
                            "",
                            false,
                        );
                    } else {
                        let _ = discord.send_message(
                            message.channel_id,
                            "not this time; a total fabrication.",
                            "",
                            false,
                        );
                    }
                    let duration = start.elapsed();
                    println!("Time elapsed sending a message: {:?}", duration);
                } else if message.content == "!quit" {
                    println!("Quitting.");
                    break;
                }
            }
            // funny bot action
            Ok(Event::ReactionAdd(reaction)) | Ok(Event::ReactionRemove(reaction)) => {
                let botcache_id = (*serverid_to_cache_map.get(channel_to_server_map.get(&reaction.channel_id).unwrap()).unwrap()).0;
                let botcache_channel_id = (*serverid_to_cache_map.get(channel_to_server_map.get(&reaction.channel_id).unwrap()).unwrap()).1;
                match Discord::get_message(&discord, reaction.channel_id, reaction.message_id) {
                    Ok(message) => {

                        let rating = get_rating(&discord, &message);
                        println!("rating: {}", rating);

                        if rating > 7.999 {


                            let message_reproduction = reproduce_message(message.clone(), rating, &channel_to_server_map);
                            if !original_to_reproduction_map.contains_key(&message.id) {
                                let sent = discord.send_message(
                                    *funny_channels_map.get(&message.channel_id).unwrap()
                                    , &message_reproduction
                                    , ""
                                    , false
                                ).unwrap();
                                original_to_reproduction_map.insert(message.id, sent.id);
                                let _ = discord.edit_message(
                                    botcache_channel_id
                                    , botcache_id
                                    , &format!(
                                        "{}\n{}:{}"
                                        , discord.get_message(
                                            botcache_channel_id
                                            , botcache_id
                                        ).unwrap().content
                                        , message.id
                                        , sent.id
                                    )
                                );
                            } else {
                                let message_reproduction = reproduce_message(message.clone(), rating, &channel_to_server_map);

                                let _ = discord.edit_message(
                                    *funny_channels_map.get(&message.channel_id).unwrap()
                                    , *original_to_reproduction_map.get(&message.id).unwrap()
                                    , &message_reproduction
                                );
                            }
                        } else {
                            if original_to_reproduction_map.contains_key(&message.id) {
                                let _ = discord.delete_message(*funny_channels_map.get(&message.channel_id).unwrap(), *original_to_reproduction_map.get(&message.id).unwrap());
                                let botcache = discord.get_message(
                                    botcache_channel_id,
                                    botcache_id
                                ).unwrap();
                                let mut old_botcache = botcache.content.split("\n").collect::<Vec<_>>();
                                old_botcache.pop();
                                let _ = discord.edit_message(
                                    botcache_channel_id
                                    , botcache_id
                                    , &old_botcache.join("\n")
                                );
                                original_to_reproduction_map.remove(&message.id);
                            }
                        }






                    }
                    Err(message) => {println!("couldn't get the message")}
                }


                //let reaction_emoji = reaction.emoji;

                let reaction_emoji;

                match reaction.emoji {
                    ReactionEmoji::Unicode(string) => reaction_emoji=string,
                    ReactionEmoji::Custom { name: name, id: _ } => reaction_emoji=name,
                }

                println!("{} added or removed {} on {}", reaction.user_id, reaction_emoji, reaction.message_id);

                let botcache = discord.get_message(botcache_channel_id, botcache_id).unwrap();
                cull_meme_cache(&discord, &botcache, MEME_CACHE_LINES);
            }
            Ok(_) => {}
            Err(discord::Error::Closed(code, body)) => {
                println!("Gateway closed on us with code {:?}: {}", code, body);
                break;
            }
            Err(err) => println!("Receive error: {:?}", err),
        }
    }
}