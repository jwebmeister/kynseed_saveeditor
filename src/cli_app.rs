use std::hash::{Hash, Hasher};
use std::time::Duration;
use std::path::PathBuf;
use std::io::Write;


use crate::config;
use crate::lootitems;
use crate::savedata;
use crate::apothrecipes;


#[derive(Debug)]
pub struct SaveInventoryItemCount {
    pub count: i32,
    // count_string: String
}

impl SaveInventoryItemCount {

    pub fn new(count: i32) -> Self {
        Self {
            count,
            // count_string: count.to_string()
        }
    }

}

pub struct AppSaveInventoryItem {
    pub save_item_ref: savedata::SaveInventoryItemRef,
    pub uid: i32,
    pub counts: [SaveInventoryItemCount; 5],
    pub name: String,
    pub pickup_type_name: String,
    pub cost: i32,
    // sprite_idx: i32
}

impl Hash for AppSaveInventoryItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.save_item_ref.key_int_node.hash(state);
    }
}

impl AppSaveInventoryItem {

    pub fn new(save_item_ref: &savedata::SaveInventoryItemRef, sm: &savedata::SaveDataManager, lm: &lootitems::LootManager) -> Self {
        let sir = save_item_ref.clone();
        let uid = save_item_ref.get_uid(sm);
        let counts = save_item_ref.get_counts(sm).map(SaveInventoryItemCount::new);
        let li = save_item_ref.get_lootitem_ref(sm,lm);
        let li_name = li.name.to_string();
        let li_pickup_type = lm.pickup_type_lookup_rev[&li.type_of_pickup].to_string();

        Self {
            save_item_ref: sir,
            uid,
            counts,
            name: li_name,
            pickup_type_name: li_pickup_type,
            cost: li.cost,
            // sprite_idx: li.sprite_idx
        }
    }
}

pub fn set_save_items_qty_800(sm: &mut savedata::SaveDataManager, lm: &lootitems::LootManager, arm: Option<&apothrecipes::ApothRecipeManager>) {
    for item in sm.save_inventory_ref.iter() {
        // println!("{:?}", sm.xtree.text_content_str(item.key_int_node));
        let uid = sm.xtree.text_content_str(item.key_int_node).unwrap().parse::<i32>().unwrap();
        // let li = &lm.full_item_lookup[&uid];
        // let li_pickup_type = &lm.pickup_type_lookup_rev[&li.type_of_pickup];
        let max_qty = lm.get_max_item_quantity(uid);
        // println!("{},{},{},{:?}", uid, li.name, li_pickup_type, max_qty);

        let mut arm_max_qty: i32 = 999;

        match arm {
            None => {},
            Some(arm) => {
                if arm.is_not_full_cure_id(uid) {
                    arm_max_qty = 0;
                }
            }
        } 

        for (idx, count_ref) in item.count_int_nodes.iter().enumerate() {
            let count_text = sm.xtree.text_content_mut(*count_ref).unwrap();
            let compare_nums = [max_qty[idx], 800, arm_max_qty];
            let new_qty = compare_nums.iter().min().unwrap();
            // println!("old qty {}, new qty {}", count_text.get(), new_qty);
            count_text.set(new_qty.to_string());
        };
    };
}

pub fn set_larders_qty_100(sm: &mut savedata::SaveDataManager, lm: &lootitems::LootManager, arm: Option<&apothrecipes::ApothRecipeManager>) {
    
    if !sm.newlarder_item_ref.is_empty() {
        for item in sm.newlarder_item_ref.iter() {
            // println!("{:?}", sm.xtree.text_content_str(item.key_int_node));
            let uid = sm.xtree.text_content_str(item.key_int_node).unwrap().parse::<i32>().unwrap();
            // let li = &lm.full_item_lookup[&uid];
            // let li_pickup_type = &lm.pickup_type_lookup_rev[&li.type_of_pickup];
            let mut max_qty = lm.get_max_item_quantity(uid).map(|x| x.min(100));
            // println!("{},{},{},{:?}", uid, li.name, li_pickup_type, max_qty);

            if max_qty[4] > 0 {
                max_qty[0..4].fill(0);
            }

            if uid == 0 {
                max_qty.fill(0);
            }

            let mut arm_max_qty: i32 = 100;

            match arm {
                None => {},
                Some(arm) => {
                    if arm.is_not_full_cure_id(uid) {
                        arm_max_qty = 0;
                    }
                }
            } 

            for (idx, count_ref) in item.count_int_nodes.iter().enumerate() {
                let count_text = sm.xtree.text_content_mut(*count_ref).unwrap();
                let compare_nums = [max_qty[idx], 100, arm_max_qty];
                let new_qty = compare_nums.iter().min().unwrap();
                // println!("homelarder old qty {}, new qty {}", count_text.get(), new_qty);
                count_text.set(new_qty.to_string());
            };
        };
    };

    if !sm.savedshops_item_ref.is_empty() {
        for item in sm.savedshops_item_ref.iter() {
            // println!("{:?}", sm.xtree.text_content_str(item.key_int_node));
            let uid = sm.xtree.text_content_str(item.key_int_node).unwrap().parse::<i32>().unwrap();
            // let li = &lm.full_item_lookup[&uid];
            // let li_pickup_type = &lm.pickup_type_lookup_rev[&li.type_of_pickup];
            let mut max_qty = lm.get_max_item_quantity(uid).map(|x| x.min(100));
            // println!("{},{},{},{:?}", uid, li.name, li_pickup_type, max_qty);

            if max_qty[4] > 0 {
                max_qty[0..4].fill(0);
            }

            if uid == 0 {
                max_qty.fill(0);
            }

            let mut arm_max_qty: i32 = 100;

            match arm {
                None => {},
                Some(arm) => {
                    if arm.is_not_full_cure_id(uid) {
                        arm_max_qty = 0;
                    }
                }
            } 

            for (idx, count_ref) in item.count_int_nodes.iter().enumerate() {
                let count_text = sm.xtree.text_content_mut(*count_ref).unwrap();
                let compare_nums = [max_qty[idx], 100, arm_max_qty];
                let new_qty = compare_nums.iter().min().unwrap();
                // println!("shops old qty {}, new qty {}", count_text.get(), new_qty);
                count_text.set(new_qty.to_string());
            };
        };
    };

}

pub fn write_savedata(appconfig: &config::AppConfig, sm: &mut savedata::SaveDataManager) {
    backup_save(appconfig).unwrap();

    let outfile_path = PathBuf::from_iter([&appconfig.path_kynseed_saves, &appconfig.filename_kynseed_save]);
    let outfile = std::fs::File::create(outfile_path).unwrap();
    let mut outwriter = std::io::BufWriter::new(outfile);
    writeln!(&mut outwriter, r#"<?xml version="1.0" encoding="utf-8"?>"#).unwrap();
    sm.xtree.write(sm.root.unwrap(), &mut outwriter).unwrap();
}

pub fn backup_save(appconfig: &config::AppConfig) -> Result<u64, std::io::Error> {
    let filepath_savegame = PathBuf::from_iter([&appconfig.path_kynseed_saves, &appconfig.filename_kynseed_save]);
    let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let current_time_string = format!("{:?}", &current_time);
    let mut filepath_new_backup_save = filepath_savegame.clone();
    filepath_new_backup_save.set_extension(format!("xml.bak.{}", current_time_string));
    
    std::fs::copy(filepath_savegame, filepath_new_backup_save)
}

pub struct CliApp {
    appconfig : config::AppConfig,
    lm: lootitems::LootManager,
    sm: savedata::SaveDataManager,
    save_inventory_items: Vec<AppSaveInventoryItem>
}

impl CliApp {

    pub fn new() -> Self {
        let config_filepath = config::get_config_filepath();
        let appconfig: config::AppConfig = confy::load_path(config_filepath.as_path()).unwrap();

        let mut lm = lootitems::LootManager::default();
        lm.clear_data();
        lm.load_data(&appconfig).unwrap();

        let mut sm = savedata::SaveDataManager::default();
        sm.load_data(&appconfig);

        let mut save_inventory_items: Vec<AppSaveInventoryItem>  = Vec::new();

        for siir in sm.save_inventory_ref.iter() {
            let sii = AppSaveInventoryItem::new(siir, &sm, &lm);
            save_inventory_items.push(sii);
        };

        save_inventory_items.sort_by(|a,b| 
            {let first = a.pickup_type_name.cmp(&b.pickup_type_name);
            let second = a.name.cmp(&b.name);
            first.then(second)}
        );

        Self {
            appconfig: appconfig,
            lm: lm,
            sm: sm,
            save_inventory_items: save_inventory_items
        }
    }

    pub fn run(&mut self) {
        for i in 0..1200 {
            dbg!(i);

            dbg!(std::mem::size_of_val(&self));
            dbg!(std::mem::size_of_val(&self.lm));
            dbg!(std::mem::size_of_val(&self.sm));
            dbg!(std::mem::size_of_val(&self.save_inventory_items));

            std::thread::sleep(Duration::from_millis(100));
        }
    }

}

