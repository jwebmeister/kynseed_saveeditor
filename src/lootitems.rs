use std::hash::{Hash, Hasher};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::PathBuf;
use std::error::Error;
use serde::{Serialize, Deserialize};

use crate::config::AppConfig;

const HAS_STAR_RATING_CONDITIONS_TXT: &str = include_str!("../saveedit_data/HasStarRatingConditions.txt");
const HIDE_QUANTITY_TXT: &str = include_str!("../saveedit_data/HideQuantity.txt");
const ITEM_LOOKUP_TXT: &str = include_str!("../saveedit_data/ItemLookup.txt");
const LIQUID_ITEMS_TXT: &str = include_str!("../saveedit_data/LiquidItems.txt");
const PICKUP_TYPE_TXT: &str = include_str!("../saveedit_data/PickupType.txt");

#[derive(Debug, Clone)]
struct LootDataError;

impl std::fmt::Display for LootDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to load loot data")
    }
}

impl Error for LootDataError {}


/// # LootItem 
/// An item as defined by EAItems.txt or AllItems.txt
#[derive(Debug, Serialize, Deserialize)]
pub struct LootItem {
    pub uid: i32,
    pub name: String,
    pub blank: Option<String>,
    pub sprite_idx: i32,
    pub is_carryable: String,
    pub growable_preset_idx: i32,
    pub type_of_pickup: i32,
    pub cost: i32,
    pub star_rating: i32,
    pub in_game_sprite_idx: i32,
    pub proverb_sprite: i32
}

impl Hash for LootItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HasStarRatingCondition {
    condition_type: String,
    compare_item: String,
    has_star_rating: i32
}

/// # LootManager
/// Contains data and lookups for all items
#[derive(Debug, Default)]
pub struct LootManager {
    pub full_item_lookup: HashMap<i32, LootItem>,
    pub name_item_lookup: HashMap<String, i32>,
    pub pickup_type_lookup: HashMap<String, i32>,
    pub pickup_type_lookup_rev: HashMap<i32, String>,
    pub liquid_item_lookup: HashSet<String>,
    pub hide_quantity_item_lookup: HashSet<String>,
    pub has_star_rating_conditions: Vec<HasStarRatingCondition>
}

impl LootManager {

    pub fn get_max_item_quantity(&self, uid: i32) -> [i32; 5] {
        let lootitem: &LootItem = &self.full_item_lookup[&uid];

        let li_pickup_type_name = &self.pickup_type_lookup_rev[&lootitem.type_of_pickup];
        let li_has_star_rating = self.has_star_rating(uid);
        let li_is_hide_quantity_item = self.is_hide_quantity_item(uid);

        if (li_has_star_rating) && !(li_is_hide_quantity_item) {
            return [999,999,999,999,999];
        };

        if (!li_has_star_rating) && !(li_is_hide_quantity_item) 
          && (["SEED", "GIFT", "VEG", "USABLE_ITEM", "EGG", "OTHER"].map(|x| x.to_string()).contains(li_pickup_type_name)) {
            return [999,0,0,0,0];
        };
        [1,0,0,0,0]
    }

    pub fn is_liquid_item(&self, uid: i32) -> bool {
        for luid in self.liquid_item_lookup.iter().map(|x| self.name_item_lookup[x]) {
            if luid == uid {
                return true;
            }
        }
        false
    }

    pub fn is_hide_quantity_item(&self, uid: i32) -> bool {
        for huid in self.hide_quantity_item_lookup.iter().map(|x| self.name_item_lookup[x]) {
            if huid == uid {
                return true;
            }
        }
        false
    }

    pub fn has_star_rating(&self, uid: i32) -> bool {
        let lootitem: &LootItem = &self.full_item_lookup[&uid];

        for condition in self.has_star_rating_conditions.iter() {
            let condition_has_star_rating: bool = condition.has_star_rating == 1;
            if condition.condition_type == *"UniqueID" {
                let c_uid: i32 = condition.compare_item.parse().unwrap_or(i32::MAX);
                if uid == c_uid {
                    return condition_has_star_rating;
                }
            };

            if condition.condition_type == *"typeOfPickup" {
                let c_pickup_type: i32 = self.pickup_type_lookup[&condition.compare_item];
                if lootitem.type_of_pickup == c_pickup_type {
                    return condition_has_star_rating;
                }
            };

            if condition.condition_type == *"namecontains" && lootitem.name.contains(&condition.compare_item) {
                return condition_has_star_rating;
            };

            if condition.condition_type == *"ItemLookup" {
                let c_uid: i32 = self.name_item_lookup[&condition.compare_item];
                if uid == c_uid {
                    return condition_has_star_rating;
                }
            };
            
            if condition.condition_type == *"isLiquidItem" && self.is_liquid_item(uid) {
                return condition_has_star_rating;
            };
        }
        true
    }


    pub fn load_data(&mut self, appconfig: &AppConfig) -> Result<(), Box<dyn Error>> {
        
        self.load_full_item_lookup(&appconfig.path_kynseed_data, &appconfig.filenames_kynseed_items)?;

        let mut filepath_name_item_lookup: PathBuf = PathBuf::from("fake_path");
        let mut filepath_pickup_types: PathBuf = PathBuf::from("fake_path");
        let mut filepath_liquid_items: PathBuf = PathBuf::from("fake_path");
        let mut filepath_hide_quantity_item_lookup: PathBuf = PathBuf::from("fake_path");
        let mut filepath_has_star_rating_conditions: PathBuf = PathBuf::from("fake_path");

        match appconfig.b_use_embedded_saveedit_data {
            false => {
                filepath_name_item_lookup = PathBuf::from_iter([&appconfig.path_saveedit_data, &appconfig.filename_saveedit_name_item_lookup]);
                filepath_pickup_types = PathBuf::from_iter([&appconfig.path_saveedit_data, &appconfig.filename_saveedit_pickup_types]);
                filepath_liquid_items = PathBuf::from_iter([&appconfig.path_saveedit_data, &appconfig.filename_saveedit_liquid_items]);
                filepath_hide_quantity_item_lookup = PathBuf::from_iter([&appconfig.path_saveedit_data, &appconfig.filename_saveedit_hide_quantity_items]);
                filepath_has_star_rating_conditions = PathBuf::from_iter([&appconfig.path_saveedit_data, &appconfig.filename_saveedit_has_star_rating_conditions]);
                let all_files_exist = [
                    filepath_name_item_lookup.clone(), 
                    filepath_pickup_types.clone(), 
                    filepath_liquid_items.clone(), 
                    filepath_hide_quantity_item_lookup.clone(), 
                    filepath_has_star_rating_conditions.clone()
                    ].iter().map(|x|x.is_file()).fold(true, |acc, el| {acc && el});
                if !all_files_exist {return Err(Box::new(LootDataError))};
            },
            true => {}
        }

        // let filepath_name_item_lookup = PathBuf::from_iter([&appconfig.path_saveedit_data, &appconfig.filename_saveedit_name_item_lookup]);
        // let filepath_name_item_lookup: PathBuf = PathBuf::from("fake_path");
        self.load_name_item_lookup(&filepath_name_item_lookup)?;

        // let filepath_pickup_types = PathBuf::from_iter([&appconfig.path_saveedit_data, &appconfig.filename_saveedit_pickup_types]);
        // let filepath_pickup_types: PathBuf = PathBuf::from("fake_path");
        self.load_pickup_type_lookup(&filepath_pickup_types)?;

        // let filepath_liquid_items = PathBuf::from_iter([&appconfig.path_saveedit_data, &appconfig.filename_saveedit_liquid_items]);
        // let filepath_liquid_items: PathBuf = PathBuf::from("fake_path");
        self.load_liquid_item_lookup(&filepath_liquid_items)?;

        // let filepath_hide_quantity_item_lookup = PathBuf::from_iter([&appconfig.path_saveedit_data, &appconfig.filename_saveedit_hide_quantity_items]);
        // let filepath_hide_quantity_item_lookup: PathBuf = PathBuf::from("fake_path");
        self.load_hide_quantity_item_lookup(&filepath_hide_quantity_item_lookup)?;

        // let filepath_has_star_rating_conditions = PathBuf::from_iter([&appconfig.path_saveedit_data, &appconfig.filename_saveedit_has_star_rating_conditions]);
        // let filepath_has_star_rating_conditions: PathBuf = PathBuf::from("fake_path");
        self.load_has_star_rating_conditions(&filepath_has_star_rating_conditions)?;

        Ok(())
    }

    pub fn clear_data(&mut self) {
        self.full_item_lookup.clear();
        self.name_item_lookup.clear();
        self.pickup_type_lookup.clear();
        self.liquid_item_lookup.clear();
        self.hide_quantity_item_lookup.clear();
        self.has_star_rating_conditions.clear();
    }
    
    pub fn load_full_item_lookup(&mut self, folder_string: &String, filenames: &[String]) -> Result<(), Box<dyn Error>> {
        self.full_item_lookup.clear();
        for filename in filenames.iter() {
            let file_path = PathBuf::from_iter([folder_string, filename]);
            self.load_kynseed_item_file(&file_path)?;
        };
        Ok(())
    }

    pub fn load_kynseed_item_file(&mut self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'|')
            .has_headers(false)
            .from_path(file_path)?;
        for result in rdr.deserialize() {
            let lootitem: LootItem = result?;
            self.full_item_lookup.insert(lootitem.uid, lootitem);
        }
        Ok(())
    }

    pub fn load_name_item_lookup(&mut self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        match file_path.is_file() {
            true => {
                let mut rdr = csv::ReaderBuilder::new()
                    .delimiter(b'|')
                    .has_headers(false)
                    .from_path(file_path)?;
                self.name_item_lookup.clear();
                for result in rdr.deserialize() {
                    let record: (String, i32) = result?;
                    self.name_item_lookup.insert(record.0, record.1);
                };
                Ok(())
            }
            false => {
                let mut rdr = csv::ReaderBuilder::new()
                    .delimiter(b'|')
                    .has_headers(false)
                    .from_reader(Cursor::new(ITEM_LOOKUP_TXT));
                self.name_item_lookup.clear();
                for result in rdr.deserialize() {
                    let record: (String, i32) = result?;
                    self.name_item_lookup.insert(record.0, record.1);
                };
                Ok(())
            }
        }
    }

    pub fn load_pickup_type_lookup(&mut self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        match file_path.is_file() {
            true => {
                let mut rdr = csv::ReaderBuilder::new()
                    .delimiter(b'|')
                    .has_headers(false)
                    .from_path(file_path)?;
                self.pickup_type_lookup.clear();
                self.pickup_type_lookup_rev.clear();
                for result in rdr.deserialize() {
                    let record: (String, i32) = result?;
                    self.pickup_type_lookup.insert(record.0, record.1);
                }
                self.pickup_type_lookup_rev = self.pickup_type_lookup.iter().map(|(k,v)| (*v, k.clone())).collect();
                Ok(())
            }
            false => {
                let mut rdr = csv::ReaderBuilder::new()
                    .delimiter(b'|')
                    .has_headers(false)
                    .from_reader(Cursor::new(PICKUP_TYPE_TXT));
                self.pickup_type_lookup.clear();
                self.pickup_type_lookup_rev.clear();
                for result in rdr.deserialize() {
                    let record: (String, i32) = result?;
                    self.pickup_type_lookup.insert(record.0, record.1);
                }
                self.pickup_type_lookup_rev = self.pickup_type_lookup.iter().map(|(k,v)| (*v, k.clone())).collect();
                Ok(())
            }
        }
    }

    pub fn load_liquid_item_lookup(&mut self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        match file_path.is_file() {
            true => {
                let mut rdr = csv::ReaderBuilder::new()
                    .delimiter(b'|')
                    .has_headers(false)
                    .from_path(file_path)?;
                self.liquid_item_lookup.clear();
                for result in rdr.deserialize() {
                    let record: String = result?;
                    self.liquid_item_lookup.insert(record);
                };
                Ok(())
            }
            false => {
                let mut rdr = csv::ReaderBuilder::new()
                    .delimiter(b'|')
                    .has_headers(false)
                    .from_reader(Cursor::new(LIQUID_ITEMS_TXT));
                self.liquid_item_lookup.clear();
                for result in rdr.deserialize() {
                    let record: String = result?;
                    self.liquid_item_lookup.insert(record);
                };
                Ok(())
            }
        }
    }

    pub fn load_hide_quantity_item_lookup(&mut self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        match file_path.is_file() {
            true => {
                let mut rdr = csv::ReaderBuilder::new()
                    .delimiter(b'|')
                    .has_headers(false)
                    .from_path(file_path)?;
                self.hide_quantity_item_lookup.clear();
                for result in rdr.deserialize() {
                    let record: String = result?;
                    self.hide_quantity_item_lookup.insert(record);
                };
                Ok(())
            }
            false => {
                let mut rdr = csv::ReaderBuilder::new()
                    .delimiter(b'|')
                    .has_headers(false)
                    .from_reader(Cursor::new(HIDE_QUANTITY_TXT));
                self.hide_quantity_item_lookup.clear();
                for result in rdr.deserialize() {
                    let record: String = result?;
                    self.hide_quantity_item_lookup.insert(record);
                };
                Ok(())
            }
        }
    }

    pub fn load_has_star_rating_conditions(&mut self, file_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        match file_path.is_file() {
            true => {
                let mut rdr = csv::ReaderBuilder::new()
                    .delimiter(b'|')
                    .has_headers(false)
                    .from_path(file_path)?;
                self.has_star_rating_conditions.clear();
                for result in rdr.deserialize() {
                    let record: HasStarRatingCondition = result?;
                    self.has_star_rating_conditions.push(record);
                };
                Ok(())
            }
            false => {
                let mut rdr = csv::ReaderBuilder::new()
                    .delimiter(b'|')
                    .has_headers(false)
                    .from_reader(Cursor::new(HAS_STAR_RATING_CONDITIONS_TXT));
                self.has_star_rating_conditions.clear();
                for result in rdr.deserialize() {
                    let record: HasStarRatingCondition = result?;
                    self.has_star_rating_conditions.push(record);
                };
                Ok(())
            }
        }
    }
}

