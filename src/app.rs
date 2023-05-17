use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::io::Write;


use egui::{DragValue};

use crate::config;
use crate::lootitems;
use crate::savedata;
use crate::apothrecipes;


pub struct App {
    appconfig : config::AppConfig,
    lm: lootitems::LootManager,
    sm: savedata::SaveDataManager,
    save_inventory_items: Vec<AppSaveInventoryItem>,
    arm: apothrecipes::ApothRecipeManager
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

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

        let mut arm = apothrecipes::ApothRecipeManager::default();
        arm.load_data(&appconfig);

        Self {
            appconfig,
            lm,
            sm,
            save_inventory_items,
            arm
        }
    }

    pub fn top_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() {
                        write_savedata(&self.appconfig, &mut self.sm);
                        ui.close_menu();
                    };
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    };
                });
                ui.menu_button("Inventory", |ui| {
                    if ui.button("Give me 800 qty!").clicked() {
                        set_save_items_qty_800(&mut self.sm, &self.lm, Some(&self.arm));
                        self.update_allitems_fromref();
                        ui.close_menu();

                    };
                    if ui.button("Give me 100 qty in larder").clicked() {
                        set_larders_qty_100(&mut self.sm, &self.lm, Some(&self.arm));
                        self.update_allitems_fromref();
                        ui.close_menu();
                    };
                    if ui.button("Sort by type, name").clicked() {
                        self.save_inventory_items.sort_by(|a,b| 
                            {let first = a.pickup_type_name.cmp(&b.pickup_type_name);
                            let second = a.name.cmp(&b.name);
                            first.then(second)}
                        );
                        ui.close_menu();
                    };
                    if ui.button("Sort by UID").clicked() {
                        self.save_inventory_items.sort_by(|a,b| 
                            {a.uid.cmp(&b.uid)}
                        );
                        ui.close_menu();
                    };
                    if ui.button("Sort by cost, name").clicked() {
                        self.save_inventory_items.sort_by(|a,b| 
                            {let first = b.cost.cmp(&a.cost);
                            let second = a.name.cmp(&b.name);
                            first.then(second)}
                        );
                        ui.close_menu();
                    };
                });
            });
        });
    }

    pub fn central_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            use egui_extras::{Column, TableBuilder};

            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                // .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .cell_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown))
                .column(Column::exact(40.0))
                .column(Column::exact(65.0))
                .column(Column::exact(65.0))
                .column(Column::exact(65.0))
                .column(Column::exact(65.0))
                .column(Column::exact(65.0))
                .column(Column::initial(160.0).range(40.0..=200.0).resizable(true))
                .column(Column::initial(140.0).range(40.0..=200.0).resizable(true))
                .column(Column::exact(40.0))
                .min_scrolled_height(0.0);

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("uid");
                    });
                    header.col(|ui| {
                        ui.strong("Qty, Star 1");
                    });
                    header.col(|ui| {
                        ui.strong("Qty, Star 2");
                    });
                    header.col(|ui| {
                        ui.strong("Qty, Star 3");
                    });
                    header.col(|ui| {
                        ui.strong("Qty, Star 4");
                    });
                    header.col(|ui| {
                        ui.strong("Qty, Star 5");
                    });
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    header.col(|ui| {
                        ui.strong("Type");
                    });
                    header.col(|ui| {
                        ui.strong("Cost");
                    });
                })
                .body(|body| {
                    let row_height = 30.0;
                    let num_rows = self.save_inventory_items.len();
                    body.rows(row_height, num_rows, |row_index, mut row| {
                        row.col(|ui| {
                            ui.label(self.save_inventory_items[row_index].uid.to_string());
                        });
                        for count_index in 0..5 {
                            row.col(|ui| {
                                let dragvalue_response = ui.add(DragValue::new(&mut self.save_inventory_items[row_index].counts[count_index].count));
                                if dragvalue_response.changed() {
                                    let new_count = self.save_inventory_items[row_index].save_item_ref.set_count_at_idx(
                                        count_index, 
                                        self.save_inventory_items[row_index].counts[count_index].count, 
                                        &mut self.sm, 
                                        Some(&self.lm));
                                        self.save_inventory_items[row_index].counts[count_index].count = new_count;
                                };
                            });
                        };
                        row.col(|ui| {
                            ui.label(self.save_inventory_items[row_index].name.clone());
                        });
                        row.col(|ui| {
                            ui.label(self.save_inventory_items[row_index].pickup_type_name.clone());
                        });
                        row.col(|ui| {
                            ui.label(self.save_inventory_items[row_index].cost.to_string());
                        });
                    })
                })
        });
    }

    pub fn update_allitems_fromref(&mut self) {
        self.save_inventory_items.iter_mut().for_each(|x| x.update_fromref(&self.sm, &self.lm));
    }
}

impl eframe::App for App {

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {  
            appconfig: _,
            lm: _,
            sm: _,
            save_inventory_items: _,
            arm: _
        } = self;
        
        self.top_panel(ctx, frame);
        self.central_panel(ctx, frame);
        frame.set_window_size(ctx.used_size());
    }
}


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

    pub fn update_fromref(&mut self, sm: &savedata::SaveDataManager, lm: &lootitems::LootManager) {
        let sir = &self.save_item_ref;
        self.uid = sir.get_uid(sm);
        self.counts = sir.get_counts(sm).map(SaveInventoryItemCount::new);
        let li = sir.get_lootitem_ref(sm,lm);
        self.name = li.name.to_string();
        self.pickup_type_name = lm.pickup_type_lookup_rev[&li.type_of_pickup].to_string();
        self.cost = li.cost;
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