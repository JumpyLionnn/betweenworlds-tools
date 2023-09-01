use std::io::{self, Write};
use betweenworlds_api::{Client, UserDataFlags, LeaderboardsFlags, Item, ConsumeEffect};
use std::env;

fn main() {
    println!("Welcome to the betweenworlds networth calculator!");

    let username = if let Some(arg) = env::args().nth(1) {
        arg
    }
    else {
        read_line("username: ").trim().to_string()
    };

    let api_key = if let Some(arg) = env::args().nth(2) {
        arg
    }
    else {
        read_line("api-key: ").trim().to_string()
    };

    let client = Client::new(username.to_string(), api_key.to_string());
    let user = client.get_user(&username, UserDataFlags::Inventory | UserDataFlags::Equipment).unwrap();
    let items_collection = client.get_items_map().unwrap();

    
    let mut total = 0;

    match user.equipment {
        Some(equipment) => {
            let mut equipment_worth = 0;
            for item in equipment {
                let item_worth_multiplier = items_collection.get(&item.item_name).expect("couldn't find item").worth_multiplier;
                let sell_price = item_worth_multiplier * (item.quality + 1) as usize;
                equipment_worth += sell_price;
            }
            println!("The equipment is worth {equipment_worth} credits.");
            total += equipment_worth;
        },
        None => {
            eprintln!("Unable to get equipment");
        },
    }

    match user.inventory {
        Some(inventory) => {
            let mut inventory_worth = 0;
            for item in inventory {
                let item_info = items_collection.get(&item.item_name).expect("couldn't find item");
                let sell_price = calculate_item_worth(item_info, item.quality) * item.quantity;
                inventory_worth += sell_price;
            }
            println!("The inventory is worth {inventory_worth} credits.");
            total += inventory_worth;
        },
        None => {
            eprintln!("Unable to get inventory");
        },
    }
    
    let leaderboards_user = client.get_leaderboard_user(&username, LeaderboardsFlags::Credits).unwrap();
    match leaderboards_user.credits {
        Some(credits) => {
            println!("The account has {} credits", credits.credits);
            total += credits.credits;
        },
        None => {
            eprintln!("Unable to get credits");
        },
    }

    println!("The account networth is {} credits.", total);
}

fn read_line(text: &str) -> String {
    print!("{text}");
    io::stdout().flush().unwrap();
    let mut text = String::new();
    io::stdin().read_line(&mut text).unwrap();
    text
}


fn calculate_item_worth(item: &Item, quality: u8) -> usize {
    let sell_value = item.worth_multiplier * (quality + 1) as usize;
    match &item.consume_effects {
        Some(effects) => {
            let mut value = 0;
            for effect in effects {
                match effect {
                    ConsumeEffect::AddCredits(effect) => {
                        value += ((effect.min + effect.max) / 2) as usize;
                    },
                    ConsumeEffect::RemoveCredits(effect) => {
                        value -= isize::max((effect.min + effect.max) / 2, 0) as usize;
                    }
                    _ => {}
                }
            }
            usize::max(value, sell_value)
        },
        None => sell_value,
    }
}