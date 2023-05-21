use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::io::Write;


use crate::config;
use crate::lootitems;
use crate::savedata;
use crate::apothrecipes;

pub struct ShowUIState {
    loot_ref_window: bool,
    loot_ref_name_filter: String,
    loot_ref_type_filter: String,
    options_window: bool,
    top_panel: bool,
    central_panel: bool,
    error_during_load: bool,
    error_msg: String,
}

impl Default for ShowUIState {
    fn default() -> Self {
        Self {
            loot_ref_window: false,
            loot_ref_name_filter: "".to_string(),
            loot_ref_type_filter: "".to_string(),
            options_window: false,
            top_panel: true,
            central_panel: true,
            error_during_load: false,
            error_msg: "".to_string(),
        }
    }
}

pub struct App {
    appconfig : config::AppConfig,
    lm: lootitems::LootManager,
    sm: savedata::SaveDataManager,
    save_inventory_items: Vec<AppSaveInventoryItem>,
    arm: apothrecipes::ApothRecipeManager,
    show_ui_state: ShowUIState,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut show_ui_state = ShowUIState::default();

        let config_filepath = config::get_config_filepath();
        let appconfig: config::AppConfig = match confy::load_path(config_filepath.as_path()) {
            Ok(cfg) => cfg,
            Err(_e) => {
                confy::store_path(config_filepath.as_path(), config::AppConfig::default()).unwrap();
                config::AppConfig::default()
            }
        };

        let mut lm = lootitems::LootManager::default();
        lm.clear_data();
        match lm.load_data(&appconfig) {
            Ok(..) => {},
            _ => {
                lm.clear_data();
                show_ui_state.error_msg.push_str("Unable to load loot data."); 
                show_ui_state.error_during_load = true;
            } 
        };

        let mut sm = savedata::SaveDataManager::default();
        sm.clear_data();
        match sm.load_data(&appconfig) {
            Ok(..) => {},
            _ => {
                sm.clear_data();
                show_ui_state.error_msg.push_str("Unable to load save data."); 
                show_ui_state.error_during_load = true;
            } 
        };

        let mut save_inventory_items: Vec<AppSaveInventoryItem>  = Vec::new();

        match show_ui_state.error_during_load {
            true => {
                show_ui_state.options_window = true;
            },
            false => {
                for siir in sm.save_inventory_ref.iter() {
                    let sii = AppSaveInventoryItem::new(siir, &sm, &lm);
                    save_inventory_items.push(sii);
                };
        
                save_inventory_items.sort_by(|a,b| 
                    {let first = a.pickup_type_name.cmp(&b.pickup_type_name);
                    let second = a.name.cmp(&b.name);
                    first.then(second)}
                );
            }
        }

        let mut arm = apothrecipes::ApothRecipeManager::default();
        match arm.load_data(&appconfig) {
            Ok(..) => {},
            _ => {
                arm.clear_data();
                show_ui_state.error_msg.push_str("Unable to load apoth recipe data, not blocking.");
            } 
        }

        if show_ui_state.error_during_load { 
            save_inventory_items.clear();
            sm.clear_data();
            lm.clear_data();
            arm.clear_data();
        };

        Self {
            appconfig,
            lm,
            sm,
            save_inventory_items,
            arm,
            show_ui_state,
        }
    }

    pub fn loot_ref_window(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use egui_extras::{Column, TableBuilder};
        egui::Window::new("Loot reference")
            .open(&mut self.show_ui_state.loot_ref_window)
            .default_width(300.0)
            .vscroll(true)
            .show(ctx, |ui| {
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(false)
                    .cell_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown))
                    .column(Column::initial(40.0).at_least(40.0))
                    .column(Column::initial(160.0).range(40.0..=200.0).resizable(true))
                    .column(Column::initial(160.0).range(40.0..=200.0).resizable(true))
                    .column(Column::initial(40.0).at_least(40.0))
                    .min_scrolled_height(0.0);

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("uid");
                        });
                        header.col(|ui| {
                            ui.text_edit_singleline(&mut self.show_ui_state.loot_ref_name_filter);
                        });
                        header.col(|ui| {
                            ui.text_edit_singleline(&mut self.show_ui_state.loot_ref_type_filter);
                        });
                        header.col(|ui| {
                            ui.strong("Cost");
                        });
                    })
                    .body(|body| {
                        let mut filtered_lm: Vec<_> = self.lm.full_item_lookup.values().cloned().collect();
                        filtered_lm.retain(|v|  {
                            self.show_ui_state.loot_ref_name_filter.is_empty() || 
                            v.name.to_lowercase().contains(&self.show_ui_state.loot_ref_name_filter.to_lowercase())
                        } );
                        filtered_lm.retain(|v|  {
                            self.show_ui_state.loot_ref_type_filter.is_empty() || 
                            self.lm.pickup_type_lookup_rev[&v.type_of_pickup].to_lowercase().contains(&self.show_ui_state.loot_ref_type_filter.to_lowercase())
                        } );
                        filtered_lm.sort_by_key(|x| x.uid);

                        let row_height = 30.0;
                        let num_rows = filtered_lm.len();
                        body.rows(row_height, num_rows, |row_index, mut row| {
                            row.col(|ui| {
                                ui.label(format!("{}", filtered_lm[row_index].uid));
                            });
                            row.col(|ui| {
                                ui.label(&filtered_lm[row_index].name);
                            });
                            row.col(|ui| {
                                ui.label(&self.lm.pickup_type_lookup_rev[&filtered_lm[row_index].type_of_pickup]);
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", filtered_lm[row_index].cost));
                            });
                        })
                    });
                });
    }

    pub fn options_window(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Options")
            .open(&mut self.show_ui_state.options_window)
            .default_width(300.0)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.horizontal(|contents| {
                    if contents.button("Save").clicked() {
                        let config_filepath = config::get_config_filepath();
                        confy::store_path(config_filepath.as_path(), self.appconfig.clone()).unwrap();
                    };
                    if contents.button("Reload").clicked() {
                        let config_filepath = config::get_config_filepath();
                        self.appconfig = confy::load_path(config_filepath.as_path()).unwrap();

                        self.show_ui_state.error_during_load = false;
                        self.show_ui_state.error_msg = "".to_string();

                        self.save_inventory_items.clear();
                        self.sm.clear_data();
                        self.lm.clear_data();
                        self.arm.clear_data();

                        match self.lm.load_data(&self.appconfig) {
                            Ok(..) => {},
                            _ => {
                                self.lm.clear_data();
                                self.show_ui_state.error_msg.push_str("Unable to load loot data."); 
                                self.show_ui_state.error_during_load = true;
                            } 
                        };

                        
                        match self.sm.load_data(&self.appconfig) {
                            Ok(..) => {},
                            _ => {
                                self.sm.clear_data();
                                self.show_ui_state.error_msg.push_str("Unable to load save data."); 
                                self.show_ui_state.error_during_load = true;
                            } 
                        };

                        match self.show_ui_state.error_during_load {
                            true => {
                                self.save_inventory_items.clear();
                            },
                            false => {
                                for siir in self.sm.save_inventory_ref.iter() {
                                    let sii = AppSaveInventoryItem::new(siir, &self.sm, &self.lm);
                                    self.save_inventory_items.push(sii);
                                };
                        
                                self.save_inventory_items.sort_by(|a,b| 
                                    {let first = a.pickup_type_name.cmp(&b.pickup_type_name);
                                    let second = a.name.cmp(&b.name);
                                    first.then(second)}
                                );
                            }
                        }

                        match self.arm.load_data(&self.appconfig) {
                            Ok(..) => {},
                            _ => {
                                self.arm.clear_data();
                                self.show_ui_state.error_msg.push_str("Unable to load apoth recipe data, not blocking.");
                            }
                        };

                        if self.show_ui_state.error_during_load { 
                            self.save_inventory_items.clear();
                            self.sm.clear_data();
                            self.lm.clear_data();
                            self.arm.clear_data();
                        };

                    };
                    if contents.button("Reset to default").clicked() {
                        self.appconfig = config::AppConfig::default();
                    };
                });

                ui.separator();

                ui.vertical(|contents| {
                    contents.add(egui::Label::new("path_kynseed_saves"));
                    contents.add(egui::TextEdit::singleline(&mut self.appconfig.path_kynseed_saves).desired_width(f32::INFINITY));
                });
                ui.vertical(|contents| {
                    contents.add(egui::Label::new("filename_kynseed_save"));
                    contents.add(egui::TextEdit::singleline(&mut self.appconfig.filename_kynseed_save).desired_width(f32::INFINITY));
                });

                ui.separator();

                ui.vertical(|contents| {
                    contents.add(egui::Label::new("path_kynseed_data"));
                    contents.add(egui::TextEdit::singleline(&mut self.appconfig.path_kynseed_data).desired_width(f32::INFINITY));
                });
                ui.vertical(|contents| {
                    contents.add(egui::Label::new("filenames_kynseed_items"));
                    contents.vertical(|vcontents| {
                        self.appconfig.filenames_kynseed_items.iter_mut().for_each(|x| {
                            vcontents.add(egui::TextEdit::singleline(x).desired_width(f32::INFINITY));
                        });
                        vcontents.horizontal(|hcontents| {
                            if hcontents.add_sized([20.0, 20.0], egui::Button::new("+")).clicked() 
                                {self.appconfig.filenames_kynseed_items.push("".to_string())};
                            if hcontents.add_sized([20.0, 20.0], egui::Button::new("-")).clicked() 
                                {self.appconfig.filenames_kynseed_items.pop();};
                        });
                    });
                });
                ui.vertical(|contents| {
                    contents.add(egui::Label::new("filename_kynseed_apothrecipes"));
                    contents.add(egui::TextEdit::singleline(&mut self.appconfig.filename_kynseed_apothrecipes).desired_width(f32::INFINITY));
                });

                ui.separator();
                
                ui.checkbox(&mut self.appconfig.b_use_embedded_saveedit_data, "Use embedded saveedit data");
                ui.vertical(|contents| {
                    contents.add(egui::Label::new("path_saveedit_data"));
                    contents.add_enabled(!self.appconfig.b_use_embedded_saveedit_data, egui::TextEdit::singleline(&mut self.appconfig.path_saveedit_data).desired_width(f32::INFINITY));
                });
                ui.vertical(|contents| {
                    contents.add(egui::Label::new("filename_saveedit_has_star_rating_conditions"));
                    contents.add_enabled(!self.appconfig.b_use_embedded_saveedit_data, egui::TextEdit::singleline(&mut self.appconfig.filename_saveedit_has_star_rating_conditions).desired_width(f32::INFINITY));
                });
                ui.vertical(|contents| {
                    contents.add(egui::Label::new("filename_saveedit_hide_quantity_items"));
                    contents.add_enabled(!self.appconfig.b_use_embedded_saveedit_data, egui::TextEdit::singleline(&mut self.appconfig.filename_saveedit_hide_quantity_items).desired_width(f32::INFINITY));
                });
                ui.vertical(|contents| {
                    contents.add(egui::Label::new("filename_saveedit_name_item_lookup"));
                    contents.add_enabled(!self.appconfig.b_use_embedded_saveedit_data, egui::TextEdit::singleline(&mut self.appconfig.filename_saveedit_name_item_lookup).desired_width(f32::INFINITY));
                });
                ui.vertical(|contents| {
                    contents.add(egui::Label::new("filename_saveedit_liquid_items"));
                    contents.add_enabled(!self.appconfig.b_use_embedded_saveedit_data, egui::TextEdit::singleline(&mut self.appconfig.filename_saveedit_liquid_items).desired_width(f32::INFINITY));
                });
                ui.vertical(|contents| {
                    contents.add(egui::Label::new("filename_saveedit_pickup_types"));
                    contents.add_enabled(!self.appconfig.b_use_embedded_saveedit_data, egui::TextEdit::singleline(&mut self.appconfig.filename_saveedit_pickup_types).desired_width(f32::INFINITY));
                });
            });
    }

    pub fn bottom_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.colored_label(egui::Color32::RED, &mut self.show_ui_state.error_msg);
        });
    }

    pub fn top_panel(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() {
                        match write_savedata(&self.appconfig, &mut self.sm) {
                            Ok(_) => self.show_ui_state.error_msg = "".to_string(),
                            Err(e) => self.show_ui_state.error_msg = format!("{}", e)
                        }
                        ui.close_menu();
                    };
                    if ui.button("Options").clicked() {
                        self.show_ui_state.options_window = !self.show_ui_state.options_window;
                        ui.close_menu();
                    };
                    if ui.button("Quit").clicked() {
                        frame.close();
                    };
                });
                ui.menu_button("Inventory", |ui| {
                    if ui.button("Loot reference").clicked() {
                        self.show_ui_state.loot_ref_window = !self.show_ui_state.loot_ref_window;
                        ui.close_menu();

                    };
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
                    if ui.button("Sort by cost, name").clicked() {
                        self.save_inventory_items.sort_by(|a,b| 
                            {let first = b.cost.cmp(&a.cost);
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
                    if ui.button("Sort by name").clicked() {
                        self.save_inventory_items.sort_by(|a,b| 
                            {a.name.cmp(&b.name)}
                        );
                        ui.close_menu();
                    };
                });
            });
        });
    }

    pub fn central_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            use egui_extras::{Column, TableBuilder};

            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown))
                .column(Column::initial(40.0).at_least(40.0))
                .column(Column::initial(65.0).at_least(65.0))
                .column(Column::initial(65.0).at_least(65.0))
                .column(Column::initial(65.0).at_least(65.0))
                .column(Column::initial(65.0).at_least(65.0))
                .column(Column::initial(65.0).at_least(65.0))
                .column(Column::initial(160.0).range(40.0..=200.0).resizable(true))
                .column(Column::initial(160.0).range(40.0..=200.0).resizable(true))
                .column(Column::initial(40.0).at_least(40.0))
                .column(Column::initial(20.0).at_least(20.0))
                .column(Column::initial(20.0).at_least(20.0))
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
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("");
                    });
                })
                .body(|body| {
                    let row_height = 30.0;
                    let num_rows = self.save_inventory_items.len();
                    body.rows(row_height, num_rows, |row_index, mut row| {
                        row.col(|ui| {
                            // ui.label(self.save_inventory_items[row_index].uid.to_string());

                            let dragvalue_response = ui.add(egui::DragValue::new(&mut self.save_inventory_items[row_index].uid));
                            if dragvalue_response.changed() {
                                let new_uid = self.save_inventory_items[row_index].save_item_ref.set_uid(
                                    &mut self.sm, 
                                    self.save_inventory_items[row_index].uid);
                                self.save_inventory_items[row_index].uid = new_uid;
                                self.save_inventory_items[row_index].name = self.lm.full_item_lookup[&new_uid].name.clone();
                                self.save_inventory_items[row_index].pickup_type_name = self.lm.pickup_type_lookup_rev[&self.lm.full_item_lookup[&new_uid].type_of_pickup].clone();
                                self.save_inventory_items[row_index].cost = self.lm.full_item_lookup[&new_uid].cost;
                            };

                            // let combo_response = egui::ComboBox::from_id_source(row_index)
                            //     .selected_text(format!("{}|{}", 
                            //         self.save_inventory_items[row_index].uid, 
                            //         self.save_inventory_items[row_index].name))
                            //     .show_ui(ui, |ui| {
                            //         for (key, val) in self.lm.full_item_lookup.iter() {
                            //             ui.selectable_value(
                            //                 &mut self.save_inventory_items[row_index].uid, 
                            //                 *key, 
                            //                 format!("{}|{}|{}", *key, val.name, self.lm.pickup_type_lookup_rev[&val.type_of_pickup])
                            //             );
                            //         };
                                    
                            //     });
                            // if combo_response.response.changed() {
                            //     self.save_inventory_items[row_index].save_item_ref.set_uid(&mut self.sm, self.save_inventory_items[row_index].uid);
                            // };

                        });
                        for count_index in 0..5 {
                            row.col(|ui| {
                                let dragvalue_response = ui.add(egui::DragValue::new(&mut self.save_inventory_items[row_index].counts[count_index].count));
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
                        row.col(|ui| {
                            if ui.add_sized([20.0, 20.0], egui::Button::new("+")).clicked() {
                                let result_copy_new = self.save_inventory_items[row_index].save_item_ref.copy_new(&mut self.sm, savedata::LocationItemRef::Inventory);
                                match result_copy_new {
                                    Ok(siir) => {
                                        self.save_inventory_items.insert(row_index, AppSaveInventoryItem::new(&siir, &self.sm, &self.lm));
                                    },
                                    Err(_e) => {
                                        self.show_ui_state.error_msg.push_str("Error unable to copy to new inventory item."); 
                                    }
                                }
                            };
                        });
                        row.col(|ui| {
                            if ui.add_sized([20.0, 20.0], egui::Button::new("-")).clicked() {
                                let result_remove = self.save_inventory_items[row_index].save_item_ref.remove(&mut self.sm, savedata::LocationItemRef::Inventory);
                                match result_remove {
                                    Ok(_) => {
                                        self.save_inventory_items.remove(row_index);
                                    },
                                    Err(_e) => {
                                        self.show_ui_state.error_msg.push_str("Error unable to remove inventory item."); 
                                    }
                                }
                            };
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
            arm: _,
            show_ui_state: _,
        } = self;
        
        if self.show_ui_state.top_panel {self.top_panel(ctx, frame)};
        if self.show_ui_state.top_panel {self.bottom_panel(ctx, frame)};
        if self.show_ui_state.central_panel {self.central_panel(ctx, frame)};
        if self.show_ui_state.options_window {self.options_window(ctx, frame)};
        if self.show_ui_state.loot_ref_window {self.loot_ref_window(ctx, frame)};

        frame.set_window_size(ctx.used_size());
    }
}


#[derive(Debug)]
pub struct SaveInventoryItemCount {
    pub count: i32,
}

impl SaveInventoryItemCount {

    pub fn new(count: i32) -> Self {
        Self {
            count,
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
                if !arm.all_cures.is_empty() && arm.is_not_full_cure_id(uid) {
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

pub fn get_dupe_uids(sm: &savedata::SaveDataManager, lir: savedata::LocationItemRef) -> Vec<i32> {
    let vec_sir = match lir {
        savedata::LocationItemRef::Inventory => {&sm.save_inventory_ref},
        savedata::LocationItemRef::NewLarder => {&sm.newlarder_item_ref},
        savedata::LocationItemRef::SavedShops => {&sm.savedshops_item_ref},
    };
    let mut inv_uids: Vec<i32> = vec_sir.iter().map(|x| x.get_uid(sm)).collect();
    inv_uids.sort();
    let mut inv_uids_duped: std::collections::HashSet<i32> = std::collections::HashSet::new();
    inv_uids.windows(2).for_each(|x| if x[0] == x[1] {inv_uids_duped.insert(x[0]);});
    Vec::from_iter(inv_uids_duped)
}
#[derive(Debug, Clone)]
pub struct DupeUIDError(String);

impl std::fmt::Display for DupeUIDError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Dupe UIDs {:?}", self.0)
    }
}

impl std::error::Error for DupeUIDError {}

pub fn check_dupe_uids(sm: &savedata::SaveDataManager) -> Result<(), DupeUIDError> {
    let inv_dupe_uids = get_dupe_uids(sm, savedata::LocationItemRef::Inventory);
    // let newlarder_dupe_uids = get_dupe_uids(sm, savedata::LocationItemRef::NewLarder);
    // let savedshops_dupe_uids = get_dupe_uids(sm, savedata::LocationItemRef::SavedShops);

    if !inv_dupe_uids.is_empty() {return Err(DupeUIDError{0:format!("Inventory dupe UIDs {:?}", inv_dupe_uids)})};
    // if !newlarder_dupe_uids.is_empty() {return Err(DupeUIDError{0:format!("NewLarder dupe UIDs {:?}", newlarder_dupe_uids)})};
    // if !savedshops_dupe_uids.is_empty() {return Err(DupeUIDError{0:format!("SavedShops dupe UIDs {:?}", savedshops_dupe_uids)})};

    Ok(())
}

pub fn write_savedata(appconfig: &config::AppConfig, sm: &mut savedata::SaveDataManager) -> Result<(), Box<dyn std::error::Error>> {
    backup_save(appconfig).unwrap();

    check_dupe_uids(sm)?;

    let outfile_path = PathBuf::from_iter([&appconfig.path_kynseed_saves, &appconfig.filename_kynseed_save]);
    let outfile = std::fs::File::create(outfile_path)?;
    let mut outwriter = std::io::BufWriter::new(outfile);
    writeln!(&mut outwriter, r#"<?xml version="1.0" encoding="utf-8"?>"#)?;
    sm.xtree.write(sm.root.unwrap(), &mut outwriter)?;

    Ok(())
}

pub fn backup_save(appconfig: &config::AppConfig) -> Result<u64, std::io::Error> {
    let filepath_savegame = PathBuf::from_iter([&appconfig.path_kynseed_saves, &appconfig.filename_kynseed_save]);
    let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let current_time_string = format!("{:?}", &current_time);
    let mut filepath_new_backup_save = filepath_savegame.clone();
    filepath_new_backup_save.set_extension(format!("xml.bak.{}", current_time_string));
    
    std::fs::copy(filepath_savegame, filepath_new_backup_save)
}