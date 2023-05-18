use std::path::PathBuf;
use serde::{Serialize, Deserialize};

pub fn get_config_filepath() -> PathBuf {
    // let exe_path = std::env::current_exe().unwrap();
    // let config_folder = exe_path.parent().unwrap();
    let config_folder = PathBuf::from("./");
    config_folder.join("saveedit_appconfig.toml")
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub path_kynseed_data: String,
    pub path_kynseed_saves: String,
    pub path_saveedit_data: String,
    pub filename_kynseed_save: String,
    pub filenames_kynseed_items: Vec<String>,
    pub filename_kynseed_apothrecipes: String,
    pub filename_saveedit_has_star_rating_conditions: String,
    pub filename_saveedit_hide_quantity_items: String,
    pub filename_saveedit_name_item_lookup: String,
    pub filename_saveedit_liquid_items: String,
    pub filename_saveedit_pickup_types: String
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            path_kynseed_data: String::from("./Data/"),
            path_kynseed_saves: String::from("./Saves/"),
            path_saveedit_data: String::from("./saveedit_data/"),
            filename_kynseed_save: String::from("Slot1_Autosave.xml"),
            filenames_kynseed_items: [String::from("EAItems.txt"), String::from("AllItems.txt")].to_vec(),
            filename_kynseed_apothrecipes: String::from("ApothRecipes.xml"),
            filename_saveedit_has_star_rating_conditions: String::from("HasStarRatingConditions.txt"),
            filename_saveedit_hide_quantity_items: String::from("HideQuantity.txt"),
            filename_saveedit_name_item_lookup: String::from("ItemLookup.txt"),
            filename_saveedit_liquid_items: String::from("LiquidItems.txt"),
            filename_saveedit_pickup_types: String::from("PickupType.txt")
        }
    }
}
