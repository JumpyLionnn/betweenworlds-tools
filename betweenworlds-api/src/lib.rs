use std::{fmt::{Write, self}, collections::HashMap};

use bitflags::bitflags;
use reqwest::{Url, blocking::Client as ReqwestClient};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;

const BASE_URL: &str = "https://api.betweenworlds.net/v1";

/// A blocking client to interact with the between worlds api
pub struct Client {
    reqwest_client: ReqwestClient,
    auth_id: String,
    api_key: String
}

impl Client {
    /// Create a new client.
    /// auth_id - your ingame name.
    /// api_key - your api key (can be obtained in the account settings).
    pub fn new(auth_id: String, api_key: String) -> Self {
        Self { reqwest_client: ReqwestClient::new(), auth_id, api_key }
    }

    /// Get a user by it's name.
    /// Can pass along data_flags to specify what data you want to get.
    pub fn get_user(&self, username: &str, data_flags: UserDataFlags) -> Result<User, ApiError> {
        let mut url = Url::parse(&format!("{BASE_URL}/users")).expect("Unable to parse base url.");
        let mut query = format!("authId={}&apiKey={}&name={}", self.auth_id, self.api_key, username);
        if data_flags.has(UserDataFlags::Biography) {
            write!(&mut query, "&biography").expect("Couldnt write to string.");
        }
        if data_flags.has(UserDataFlags::Equipment) {
            write!(&mut query, "&equipment").expect("Couldnt write to string.");
        }
        if data_flags.has(UserDataFlags::Inventory) {
            write!(&mut query, "&inventory").expect("Couldnt write to string.");
        }
        url.set_query(Some(&query));
        self.get(url)
    }

    /// Get all the items in the game
    pub fn get_items(&self) -> Result<Vec<Item>, ApiError> {
        let url = Url::parse_with_params(
            &format!("{BASE_URL}/items"), 
            [("authId", &self.auth_id), ("apiKey", &self.api_key)]
        ).expect("Unable to parse base url.");
        self.get(url)
    }

    /// Get all the items in the game mapped by their name
    pub fn get_items_map(&self) -> Result<HashMap<String, Item>, ApiError> {
        let mut map = HashMap::new();
        for item in self.get_items()? {
            map.insert(item.name.clone(), item);
        }
        Ok(map)
    }

    /// Get the leaderboards data
    pub fn get_leaderboards(&self, data_flags: LeaderboardsFlags) -> Result<Leaderboards, ApiError> {
        let mut url = Url::parse(&format!("{BASE_URL}/leaderboards")).expect("Unable to parse base url.");
        let query = format!(
            "authId={}&apiKey={}&{}", 
            self.auth_id, 
            self.api_key, 
            self.leaderboard_flags_to_query(data_flags)
        );
        url.set_query(Some(&query));
        self.get(url)
    }

    /// get a specific user from the leaderboards
    pub fn get_leaderboard_user(&self, username: &str, data_flags: LeaderboardsFlags) -> Result<LeaderboardUser, ApiError> {
        let mut url = Url::parse(&format!("{BASE_URL}/leaderboards")).expect("Unable to parse base url.");
        let query = format!(
            "authId={}&apiKey={}&name={}&{}", 
            self.auth_id, 
            self.api_key, 
            username,
            self.leaderboard_flags_to_query(data_flags)
        );
        url.set_query(Some(&query));
        let leaderboards: Leaderboards = self.get(url)?;
        let mut user = LeaderboardUser::default();
        // TODO: Refactor that
        if let Some(mut credits) = leaderboards.credits {
            let credits = credits.pop().ok_or(ApiError::NotFound)?;
            user.name = credits.name;
            user.roles = credits.roles;
            user.credits = Some(LeaderboardUserCredits {
                rank: credits.rank,
                credits: credits.value,
            });
        }
        if let Some(mut level) = leaderboards.highest_levels {
            let highest_levels = level.pop().ok_or(ApiError::NotFound)?;
            user.name = highest_levels.name;
            user.roles = highest_levels.roles;
            user.highest_levels = Some(LeaderboardUserLevel { 
                rank: highest_levels.rank, 
                level: highest_levels.level, 
                exp_current: highest_levels.exp_current
            });
        }
        if let Some(mut combats_won) = leaderboards.combats_won {
            let combats_won = combats_won.pop().ok_or(ApiError::NotFound)?;
            user.name = combats_won.name;
            user.roles = combats_won.roles;
            user.combats_won = Some(LeaderboardUserCombatsWon {
                rank: combats_won.rank,
                combats_won: combats_won.value,
            });
        }
        if let Some(mut items_crafted) = leaderboards.items_crafted {
            let items_crafted = items_crafted.pop().ok_or(ApiError::NotFound)?;
            user.name = items_crafted.name;
            user.roles = items_crafted.roles;
            user.items_crafted = Some(LeaderboardUserItemsCrafted {
                rank: items_crafted.rank,
                items_crafted: items_crafted.value,
            });
        }
        if let Some(mut jobs_performed) = leaderboards.jobs_performed {
            let jobs_performed = jobs_performed.pop().ok_or(ApiError::NotFound)?;
            user.name = jobs_performed.name;
            user.roles = jobs_performed.roles;
            user.jobs_performed = Some(LeaderboardUserJobsPerformed {
                rank: jobs_performed.rank,
                jobs_performed: jobs_performed.value,
            });
        }
        if let Some(mut overdoses) = leaderboards.overdoses {
            let overdoses = overdoses.pop().ok_or(ApiError::NotFound)?;
            user.name = overdoses.name;
            user.roles = overdoses.roles;
            user.overdoses = Some(LeaderboardUserOverdoses {
                rank: overdoses.rank,
                overdoses: overdoses.value,
            });
        }
        if let Some(mut missions_completed) = leaderboards.missions_completed {
            let missions_completed = missions_completed.pop().ok_or(ApiError::NotFound)?;
            user.name = missions_completed.name;
            user.roles = missions_completed.roles;
            user.missions_completed = Some(LeaderboardUserMissionsCompleted {
                rank: missions_completed.rank,
                missions_completed: missions_completed.value,
            });
        }
        Ok(user)
    }

    fn leaderboard_flags_to_query(&self, flags: LeaderboardsFlags) -> String {
        let mut query = DelimiterStringWriter::new("&");
        // TODO: Refactor that
        if flags.has(LeaderboardsFlags::Credits) {
            query.add("credits").expect("Couldnt write to string.");
        }
        if flags.has(LeaderboardsFlags::HighestLevels) {
            query.add("highestLevels").expect("Couldnt write to string.");
        }
        if flags.has(LeaderboardsFlags::CombatsWon) {
            query.add("combatsWon").expect("Couldnt write to string.");
        }
        if flags.has(LeaderboardsFlags::ItemsCrafted) {
            query.add("itemsCrafted").expect("Couldnt write to string.");
        }
        if flags.has(LeaderboardsFlags::JobsPerformed) {
            query.add("jobsPerformed").expect("Couldnt write to string.");
        }
        if flags.has(LeaderboardsFlags::Overdoses) {
            query.add("overdoses").expect("Couldnt write to string.");
        }
        query.get()
    }

    fn get<T: DeserializeOwned>(&self, url: Url) -> Result<T, ApiError> {
        match self.reqwest_client.get(url).send() {
            Ok(response) => {
                match response.error_for_status() {
                    Ok(response) => {
                        let text = response.text().or_else(|_| {Err(ApiError::Other)})?;
                        let user = serde_json::from_str::<T>(&text).or_else(|error| {Err(ApiError::Deserialization(error))})?;
                        Ok(user)
                    },
                    Err(error) => Err(self.get_error(error))
                }
                
            },
            Err(error) => Err(self.get_error(error))
        }
    }

    fn get_error(&self, error: reqwest::Error) -> ApiError {
        if error.is_timeout() {
            ApiError::RequestTimeout
        }
        else if let Some(status) = error.status() {
            match status.as_u16() {
                401 => ApiError::Unauthorized,
                404 => ApiError::NotFound,
                _ => ApiError::Other
            }
        }
        else {
            ApiError::Other
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct UserDataFlags: u32 {
        const Biography = 0b00000001;
        const Equipment = 0b00000010;
        const Inventory = 0b00000100;
    }
}

impl UserDataFlags {
    pub fn has(&self, flags: Self) -> bool {
        0 < (flags & *self).bits() as u32
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct LeaderboardsFlags: u32 {
        const Credits = 0b00000001;
        const HighestLevels = 0b00000010;
        const CombatsWon = 0b00000100;
        const ItemsCrafted = 0b00001000;
        const JobsPerformed = 0b00010000;
        const Overdoses = 0b00100000;
        const MissionsCompleted = 0b01000000;
    }
}

impl LeaderboardsFlags {
    pub fn has(&self, flags: Self) -> bool {
        0 < (flags & *self).bits() as u32
    }
}

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    RequestTimeout,
    Unauthorized,
    Deserialization(serde_json::Error),
    Other
}


#[derive(Debug, Deserialize)]
pub struct User {
    pub biography: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub equipment: Option<Vec<EquipmentItemRef>>,
    pub inventory: Option<Vec<InventoryItem>>,
    pub roles: Vec<String>,
    pub name: String
}

#[derive(Debug, Deserialize)]
pub struct EquipmentItemRef {
    #[serde(rename = "itemName")]
    pub item_name: String,
    pub quality: u8
}

#[derive(Debug, Deserialize)]
pub struct InventoryItem {
    #[serde(rename = "itemName")]
    pub item_name: String,
    #[serde(rename = "moduleSlots")]
    pub module_slots: u8,
    pub quality: u8,
    pub modules: Vec<serde_json::Value>,
    pub quantity: usize
}

#[derive(Debug, Deserialize)]
pub struct Item {
    #[serde(rename = "qualityAdjectives")]
    pub quality_adjectives: [String; 5],
    pub level: usize,
    #[serde(rename = "imageUrl")]
    pub image_url: String,
    #[serde(rename = "type")]
    pub item_type: usize,
    pub name: String,
    #[serde(rename = "worthMultiplier")]
    pub worth_multiplier: usize,
    #[serde(rename = "consumeEffects")]
    pub consume_effects: Option<Vec<ConsumeEffect>>,
    #[serde(rename = "skillEffects")]
    pub skill_effects: Option<Vec<serde_json::Value>>,
    #[serde(rename = "qualityDescriptions")]
    pub quality_descriptions: [String; 5]
}

use serde_enums::SerdeEnum;

#[derive(Debug, SerdeEnum)]
#[repr(u8)]
pub enum ConsumeEffect {
    Unknown = 0,
    RestoreHealth(MinMax),
    RestoreEnergy(MinMax),
    RestoreSpirit(MinMax),
    BuffHealth(MinMax),
    BuffEnergy(MinMax),
    BuffSpirit(MinMax),
    DecreaseHealth(MinMax),
    DecreaseEnergy(MinMax),
    DecreaseSpirit(MinMax),
    DebuffHealth(MinMax),
    DebuffEnergy(MinMax),
    DebuffSpirit(MinMax),
    AddCredits(AddCreditsEffect),
    RemoveCredits(MinMax),
    AcceptMission(AcceptMissionEffect),
    AddItem(AddItemEffect),
}

#[derive(Debug, Deserialize)]
pub struct MinMax {
    pub min: isize,
    pub max: isize
}

#[derive(Debug, Deserialize)]
pub struct AddCreditsEffect {
    pub min: isize,
    pub max: isize,
    #[serde(rename="worthMultiplier")]
    pub worth_multiplier: Option<usize>
}

#[derive(Debug, Deserialize)]
pub struct AcceptMissionEffect {
    #[serde(rename="missionName")]
    pub mission_name: String
}

#[derive(Debug, Deserialize)]
pub struct AddItemEffect {
    pub chance: f32,
    #[serde(rename="itemName")]
    pub item_name: String,
    #[serde(rename="qualityMin")]
    pub quality_min: u8,
    #[serde(rename="qualityMax")]
    pub quality_max: u8,
    #[serde(rename="quantityMin")]
    pub quantity_min: usize,
    #[serde(rename="quantityMax")]
    pub quantity_max: usize
}

#[derive(Debug, Deserialize)]
pub struct Leaderboards {
    pub credits: Option<Vec<LeaderboardsEntry>>,
    #[serde(rename = "highestLevels")]
    pub highest_levels: Option<Vec<LeaderboardsHighestLevelEntry>>,
    #[serde(rename = "combatsWon")]
    pub combats_won: Option<Vec<LeaderboardsEntry>>,
    #[serde(rename = "itemsCrafted")]
    pub items_crafted: Option<Vec<LeaderboardsEntry>>,
    #[serde(rename = "jobsPerformed")]
    pub jobs_performed: Option<Vec<LeaderboardsEntry>>,
    pub overdoses: Option<Vec<LeaderboardsEntry>>,
    #[serde(rename = "missionsCompleted")]
    pub missions_completed: Option<Vec<LeaderboardsEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardsEntry {
    pub rank: usize,
    #[serde(alias = "credits")]
    #[serde(alias = "level")]
    #[serde(alias = "combatsWon")]
    #[serde(alias = "itemsCrafted")]
    #[serde(alias = "jobsPerformed")]
    #[serde(alias = "overdoses")]
    #[serde(alias = "missionsCompleted")]
    pub value: usize,
    pub name: String,
    pub roles: Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct LeaderboardsHighestLevelEntry {
    pub rank: usize,
    pub level: usize,
    #[serde(rename="expCurrent")]
    pub exp_current: usize,
    pub name: String,
    pub roles: Vec<String>
}

#[derive(Debug, Default)]
pub struct LeaderboardUser {
    name: String,
    roles: Vec<String>,
    pub credits: Option<LeaderboardUserCredits>,
    pub highest_levels: Option<LeaderboardUserLevel>,
    pub combats_won: Option<LeaderboardUserCombatsWon>,
    pub items_crafted: Option<LeaderboardUserItemsCrafted>,
    pub jobs_performed: Option<LeaderboardUserJobsPerformed>,
    pub overdoses: Option<LeaderboardUserOverdoses>,
    pub missions_completed: Option<LeaderboardUserMissionsCompleted>,
}

#[derive(Debug)]
pub struct LeaderboardUserCredits {
    pub rank: usize,
    pub credits: usize
}

#[derive(Debug)]
pub struct LeaderboardUserLevel {
    pub rank: usize,
    pub level: usize,
    pub exp_current: usize
}

#[derive(Debug)]
pub struct LeaderboardUserCombatsWon {
    pub rank: usize,
    pub combats_won: usize
}
#[derive(Debug)]
pub struct LeaderboardUserItemsCrafted {
    pub rank: usize,
    pub items_crafted: usize
}
#[derive(Debug)]
pub struct LeaderboardUserJobsPerformed {
    pub rank: usize,
    pub jobs_performed: usize
}
#[derive(Debug)]
pub struct LeaderboardUserOverdoses {
    pub rank: usize,
    pub overdoses: usize
}
#[derive(Debug)]
pub struct LeaderboardUserMissionsCompleted {
    pub rank: usize,
    pub missions_completed: usize
}

struct DelimiterStringWriter<'a> {
    string: String,
    delimiter: &'a str
}

impl<'a> DelimiterStringWriter<'a> {
    pub fn new(delimiter: &'a str) -> Self {
        Self {
            string: String::new(),
            delimiter
        }
    }

    pub fn add(&mut self, element: &str) -> fmt::Result {
        if self.string.len() > 0 {
            write!(&mut self.string, "{}{}", self.delimiter, element)?;
        }
        else {
            write!(&mut self.string, "{}", element)?;
        }

        Ok(())
    }

    pub fn get(self) -> String {
        self.string
    }
}