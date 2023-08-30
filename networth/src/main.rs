use std::io::{self, Write};
use betweenworlds_api::{Client, UserDataFlags, LeaderboardsFlags};
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
            let mut equipement_worth = 0;
            for item in equipment {
                let item_worth_multiplier = items_collection.get(&item.item_name).expect("couldnt find item").worth_multiplier;
                let sell_price = item_worth_multiplier * (item.quality + 1) as usize;
                equipement_worth += sell_price;
            }
            println!("The equipement is worth {equipement_worth} credits.");
            total += equipement_worth;
        },
        None => {
            eprintln!("Unable to get equipement");
        },
    }

    match user.inventory {
        Some(inventory) => {
            let mut inventory_worth = 0;
            for item in inventory {
                let item_worth_multiplier = items_collection.get(&item.item_name).expect("couldnt find item").worth_multiplier;
                let sell_price = item_worth_multiplier * (item.quality + 1) as usize;
                inventory_worth += sell_price;
            }
            println!("The inventory is worth {inventory_worth} credits.");
            total += inventory_worth;
        },
        None => {
            eprintln!("Unable to get inventory");
        },
    }
    
    let leaderboards_user = client.get_leaderboard_user("JumpyLionnn", LeaderboardsFlags::Credits).unwrap();
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
