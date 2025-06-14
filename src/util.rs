extern crate discord;

use discord::model::{Event, Reaction, ReactionEmoji, ChannelId, ServerId,Message};
use discord::Discord;
use discord::Connection;
use std::env;
use std::time::Instant;
use std::vec;

use std::collections::HashMap;
use std::collections::HashSet;



pub fn get_rating(discord:&Discord, message:&Message) -> f32 {

    let mut emoji_value_map = HashMap::new();
    emoji_value_map.insert("0️⃣", 0 ); //0
    emoji_value_map.insert("1️⃣", 1 ); //1
    emoji_value_map.insert("2️⃣", 2 ); //2
    emoji_value_map.insert("3️⃣", 3 ); //3
    emoji_value_map.insert("4️⃣", 4 ); //4
    emoji_value_map.insert("5️⃣", 5 ); //5
    emoji_value_map.insert("6️⃣", 6 ); //6
    emoji_value_map.insert("7️⃣", 7 ); //7
    emoji_value_map.insert("8️⃣", 8 ); //8
    emoji_value_map.insert("9️⃣", 9 ); //9
    emoji_value_map.insert("🔟", 10 ); //10
    emoji_value_map.insert("🤭", 7); //hand over mouth
    emoji_value_map.insert("😆", 8); //laughing
    emoji_value_map.insert("😂", 9); //joy
    emoji_value_map.insert("🤣", 10); //rofl
    emoji_value_map.insert("💀", 10); //skull


    let mut users_who_rated = HashSet::new();
    let mut user_ratings_map = HashMap::new();

    //let mut ratings = Vec::new();

    let OP = &message.author;
    for r in message.reactions.clone() {

        //let mut all_emoji:String = String::new();
        let reaction_emoji:String;
        match r.emoji.clone() {
            ReactionEmoji::Unicode(string) => reaction_emoji=string,
            ReactionEmoji::Custom { name: name, id: _ } => reaction_emoji=name,
        }
        if emoji_value_map.contains_key(reaction_emoji.as_str()) {
            let rating= *emoji_value_map.get(reaction_emoji.as_str()).unwrap();
            let mut users = discord
                .get_reactions(
                    message.channel_id,
                    message.id,
                    r.emoji,
                    Some(50), // Max users per request (adjust as needed, max 100)
                    None, // No 'after' for pagination initially
                ).unwrap();
            for u in &users {
                if u.id==OP.id {
                    continue;
                }
                user_ratings_map.insert(u.id, rating);
                users_who_rated.insert(u.id);
            }
        }
    }
    let mut ratings = Vec::new();
    for u in users_who_rated {
        ratings.push(*user_ratings_map.get(&u).unwrap());
    }
    let mut rating = 0.0;
    if (ratings.len()>1) {
        rating = ratings.iter().sum::<i32>() as f32 / ratings.len() as f32;
        /*if ratings.len() < 3 {
            rating = ratings.iter().sum::<i32>() as f32 / ratings.len() as f32;
        } else {
            ratings.sort();
            rating = (ratings.iter().sum::<i32>() - *ratings.first().unwrap()) as f32 / (ratings.len()-1) as f32;
        }*/
    }
    return rating;
}

pub fn cull_meme_cache(discord:&Discord, meme_cache:&Message, max_len:usize, ) {
    if &meme_cache.content.split("\n").collect::<Vec<_>>().len()>&max_len {

        let extras = &meme_cache.content.split("\n").collect::<Vec<_>>().len()-(max_len);

        let old_cache = &meme_cache.content;
        let mut new_cache = String::new();
        let i = 0;
        for line in old_cache.split("\n").collect::<Vec<_>>() {
            if i==0 {
                new_cache = new_cache + line; continue
            }
            if i>extras {
                new_cache = new_cache + line;
            }
        }
        let _ = discord.edit_message(meme_cache.channel_id, meme_cache.id, &new_cache);
    }
}

pub fn reproduce_message(original:Message, rating:f32, channel_to_server_map:&HashMap<ChannelId, ServerId>) -> String {

    let mut all_attachment_urls:String = String::new();
    for att in original.attachments {
        all_attachment_urls = format!("{} {}", all_attachment_urls, att.url);
    }

    let og_message_link = format!("https://discord.com/channels/{}/{}/{}", channel_to_server_map.get(&original.channel_id).unwrap(), original.channel_id, original.id);

    if original.content.trim() == "" {
        return format!("Posted by: {}\nLink: {}\nAvg rating: {:.1}\n————————\n{}", original.author.name, og_message_link, rating, &all_attachment_urls)
    }
    format!("Posted by: {}\nLink: {}\nAvg rating: {:.1}\n————————\n{}\n{}", original.author.name, og_message_link, rating, original.content, &all_attachment_urls)
}
