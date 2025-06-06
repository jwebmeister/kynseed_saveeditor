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
    player_data_window: bool,

    save_tree_window: bool,
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
            player_data_window: false,

            save_tree_window: false,
        }
    }
}

#[derive(Default)]
pub struct PlayerData {
    brass: u32,
    stats: Vec<(usize, String, u8)>,
    tool_level: Vec<(usize, String, u8, f32)>,
}

pub struct App {
    appconfig : config::AppConfig,
    lm: lootitems::LootManager,
    sm: savedata::SaveDataManager,
    save_inventory_items: Vec<AppSaveInventoryItem>,
    arm: apothrecipes::ApothRecipeManager,
    show_ui_state: ShowUIState,
    player_data: PlayerData,
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
        let mut player_data: PlayerData = PlayerData::default();

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

                player_data.brass = {
                    match sm.xtree.text_content_str(sm.brass_count_node.unwrap()) {
                        Some(x) => x.parse().unwrap_or_else(|_| {
                            sm.clear_data();
                            show_ui_state.error_msg.push_str("Unable to load save data. Brass."); 
                            show_ui_state.error_during_load = true;
                            0
                            }),
                        _ => {
                            sm.clear_data();
                            show_ui_state.error_msg.push_str("Unable to load save data. Brass."); 
                            show_ui_state.error_during_load = true;
                            0
                        }
                    }
                };

                for (idx, item) in sm.stats_nodes.iter().enumerate() {
                    let stat_name = sm.get_name_from_node(*item);
                    let stat_val_str = sm.xtree.text_content_str(*item);
                    if stat_name.is_some() && stat_val_str.is_some() {
                        let stat_val: u8 = stat_val_str.unwrap().parse().unwrap_or_else(|_| {0});
                        player_data.stats.push((idx, stat_name.unwrap().to_string(), stat_val));
                    }
                };

                for (idx, item) in sm.tool_level_ref.iter().enumerate() {
                    let tool_name = sm.xtree.text_content_str(item.tool_type_node);
                    let tool_level_val_str = sm.xtree.text_content_str(item.tool_level_node);
                    let tool_xp_val_str = sm.xtree.text_content_str(item.tool_current_xp_node);
                    if tool_name.is_some() && tool_level_val_str.is_some() && tool_xp_val_str.is_some() {
                        let tool_level_val: u8 = tool_level_val_str.unwrap().parse().unwrap_or_else(|_| {0});
                        let tool_xp_val: f32 = tool_xp_val_str.unwrap().parse().unwrap_or_else(|_| {0.0});
                        player_data.tool_level.push((idx, tool_name.unwrap().to_string(), tool_level_val, tool_xp_val));
                    }
                };
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
            player_data = PlayerData::default();
        };

        Self {
            appconfig,
            lm,
            sm,
            save_inventory_items,
            arm,
            show_ui_state,
            player_data,
        }
    }

    pub fn reload_data_helper(&mut self) {
        Self::reload_data(&mut self.appconfig, &mut self.lm, &mut self.sm, &mut self.save_inventory_items, 
            &mut self.arm, &mut self.show_ui_state.error_during_load, &mut self.show_ui_state.error_msg, 
            &mut self.player_data);
    }

    pub fn reload_data(appconfig: &mut config::AppConfig, lm: &mut lootitems::LootManager, sm: &mut savedata::SaveDataManager, 
        save_inventory_items: &mut Vec<AppSaveInventoryItem>, arm: &mut apothrecipes::ApothRecipeManager, 
        show_ui_state_error_during_load: &mut bool, show_ui_state_error_msg: &mut String,
        player_data: &mut PlayerData
    ) {
        let config_filepath = config::get_config_filepath();
        *appconfig = match confy::load_path(config_filepath.as_path()) {
            Ok(cfg) => cfg,
            Err(_e) => {
                confy::store_path(config_filepath.as_path(), config::AppConfig::default()).unwrap();
                config::AppConfig::default()
            }
        };

        *show_ui_state_error_during_load = false;
        *show_ui_state_error_msg = "".to_string();

        save_inventory_items.clear();
        sm.clear_data();
        lm.clear_data();
        arm.clear_data();
        *player_data = PlayerData::default();

        match lm.load_data(&appconfig) {
            Ok(..) => {},
            _ => {
                lm.clear_data();
                show_ui_state_error_msg.push_str("Unable to load loot data."); 
                *show_ui_state_error_during_load = true;
            } 
        };

        match sm.load_data(&appconfig) {
            Ok(..) => {},
            _ => {
                sm.clear_data();
                show_ui_state_error_msg.push_str("Unable to load save data."); 
                *show_ui_state_error_during_load = true;
            } 
        };

        match *show_ui_state_error_during_load {
            true => {},
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

                player_data.brass = {
                    match sm.xtree.text_content_str(sm.brass_count_node.unwrap()) {
                        Some(x) => x.parse().unwrap_or_else(|_| {0}),
                        _ => {0}
                    }
                };

                for (idx, item) in sm.stats_nodes.iter().enumerate() {
                    let stat_name = sm.get_name_from_node(*item);
                    let stat_val_str = sm.xtree.text_content_str(*item);
                    if stat_name.is_some() && stat_val_str.is_some() {
                        let stat_val: u8 = stat_val_str.unwrap().parse().unwrap_or_else(|_| {0});
                        player_data.stats.push((idx, stat_name.unwrap().to_string(), stat_val));
                    }
                };

                for (idx, item) in sm.tool_level_ref.iter().enumerate() {
                    let tool_name = sm.xtree.text_content_str(item.tool_type_node);
                    let tool_level_val_str = sm.xtree.text_content_str(item.tool_level_node);
                    let tool_xp_val_str = sm.xtree.text_content_str(item.tool_current_xp_node);
                    if tool_name.is_some() && tool_level_val_str.is_some() && tool_xp_val_str.is_some() {
                        let tool_level_val: u8 = tool_level_val_str.unwrap().parse().unwrap_or_else(|_| {0});
                        let tool_xp_val: f32 = tool_xp_val_str.unwrap().parse().unwrap_or_else(|_| {0.0});
                        player_data.tool_level.push((idx, tool_name.unwrap().to_string(), tool_level_val, tool_xp_val));
                    }
                };
            }
        }

        match arm.load_data(&appconfig) {
            Ok(..) => {},
            _ => {
                arm.clear_data();
                show_ui_state_error_msg.push_str("Unable to load apoth recipe data, not blocking.");
            } 
        }

        if *show_ui_state_error_during_load { 
            save_inventory_items.clear();
            sm.clear_data();
            lm.clear_data();
            arm.clear_data();
            *player_data = PlayerData::default();
        };
    }

    pub fn save_tree_child_ui(ui: &mut egui::Ui, item: &mut savedata::SaveNodeTree, xtree: &mut xot::Xot,
        siir: &mut Vec<AppSaveInventoryItem>, lm: &lootitems::LootManager, // todo: remove invtree, appsaveitem, playerdata coupling
        player_data: &mut PlayerData, brass_count_node: &Option<xot::Node>, stats_nodes:&Vec<xot::Node>, tool_level_ref: &Vec<savedata::ToolLevelRef>,
        show_ui_state_error_msg: &mut String, saveinvref: &mut Vec<savedata::SaveInventoryItemRef>
        )
    {
        let id = ui.make_persistent_id(item.0);
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false)
            .show_header(ui, |ui| {
                ui.label(&item.1);
            })
            .body(|body| {
                if item.3 {
                    if body.text_edit_singleline(&mut item.2).changed() {
                        item.set_str_from_self(xtree);
                        siir.iter_mut().for_each(|x| x.update_fromref_xt(xtree, lm)); // todo: remove invtree, appsaveitem, playerdata coupling
                        Self::update_playerdata(player_data, xtree, brass_count_node, stats_nodes, tool_level_ref);
                    };
                };
                
                let child_row_num = item.4.len();
                for child_idx in 0..child_row_num {
                    let mut child_node_deref: Option<xot::Node> = None;
                    if let Some(child) = item.4.get_mut(child_idx) {
                        child_node_deref = Some(child.0.clone());

                        Self::save_tree_child_ui(body, child, xtree, siir, lm, player_data, brass_count_node, stats_nodes, tool_level_ref, 
                            show_ui_state_error_msg, saveinvref
                        );
                    };

                    Self::tree_modify_ui(body, child_node_deref, item, xtree, 
                        siir, lm, 
                        player_data, brass_count_node, stats_nodes, tool_level_ref,
                        show_ui_state_error_msg, saveinvref);
                };
                
            });
    }

    pub fn tree_modify_ui(ui: &mut egui::Ui, child_node_deref: Option<xot::Node>, item: &mut savedata::SaveNodeTree, xtree: &mut xot::Xot,
        siir: &mut Vec<AppSaveInventoryItem>, lm: &lootitems::LootManager, // todo: remove invtree, appsaveitem, playerdata coupling
        player_data: &mut PlayerData, brass_count_node: &Option<xot::Node>, stats_nodes:&Vec<xot::Node>, tool_level_ref: &Vec<savedata::ToolLevelRef,>,
        show_ui_state_error_msg: &mut String,  saveinvref: &mut Vec<savedata::SaveInventoryItemRef>
        ) 
    {
        ui.horizontal(|body| {
            body.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |body| {
                if let Some(node_ref) = child_node_deref {
                    if body.add_sized([20.0, 20.0], egui::Button::new("+")).clicked() {
                        match savedata::SaveNodeTree::copy_node(item, xtree, &node_ref) {
                            Ok(new_node) => {
                                let mut siir_idx_match: Option<usize> = None;
                                let mut siir_item_match: Option<&AppSaveInventoryItem> = None;
                                for (siir_idx, siir_item) in siir.iter().enumerate() {
                                    if node_ref == siir_item.save_item_ref.item_node {
                                        siir_idx_match = Some(siir_idx);
                                        siir_item_match = Some(siir_item);
                                        break
                                    };
                                };
                                if let Some(siir_idx_match) = siir_idx_match {
                                    match savedata::SaveDataManager::get_sir_from_item_node(new_node, xtree) {
                                        Ok(si) => {
                                            let asii = AppSaveInventoryItem::new_xt(&si, xtree, lm );
                                            saveinvref.push(si);
                                            siir.insert(siir_idx_match, asii);
                                        },
                                        Err(_e) => *show_ui_state_error_msg = "Could not add item to UI, please save and reload.".to_string(),
                                    }
                                };
                            },
                            Err(s) => {
                                *show_ui_state_error_msg = s
                            },
                        };
                    };
                    if body.add_sized([20.0, 20.0], egui::Button::new("-")).clicked() {
                        match savedata::SaveNodeTree::remove_node(item, xtree, &node_ref) {
                            Ok(_) => {
                                let mut siir_idx_match: Option<usize> = None;
                                let mut siir_item_match: Option<&AppSaveInventoryItem> = None;
                                for (siir_idx, siir_item) in siir.iter().enumerate() {
                                    if node_ref == siir_item.save_item_ref.item_node {
                                        siir_idx_match = Some(siir_idx);
                                        siir_item_match = Some(siir_item);
                                        break
                                    };
                                };
                                if let (Some(siir_idx_match), Some(siir_item_match)) = (siir_idx_match, siir_item_match) {
                                    let mut si_idx_match: Option<usize> = None;
                                    for (si_idx, si) in saveinvref.iter().enumerate() {
                                        if *si == siir_item_match.save_item_ref {
                                            si_idx_match = Some(si_idx);
                                        };
                                    };
                                    if let Some(si_idx_match) = si_idx_match {
                                        saveinvref.remove(si_idx_match);
                                    };
                                    
                                    siir.remove(siir_idx_match);
                                }
                            },
                            Err(s) => *show_ui_state_error_msg = s,
                        };
                    };
                };
            });
        });
    }

    pub fn save_tree_window(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Save Tree")
            .open(&mut self.show_ui_state.save_tree_window)
            .default_width(300.0)
            .vscroll(true)
            .show(ctx, |ui| {
                if let Some(item) = &mut self.sm.save_tree {
                    Self::save_tree_child_ui(ui, item, &mut self.sm.xtree, 
                        &mut self.save_inventory_items,  &self.lm, 
                        &mut self.player_data, &self.sm.brass_count_node, &self.sm.stats_nodes, &self.sm.tool_level_ref, 
                        &mut self.show_ui_state.error_msg, &mut self.sm.save_inventory_ref
                    ); // todo: remove invtree, appsaveitem, playerdata coupling
                };
            });
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
                        body.rows(row_height, num_rows, |mut row| {
                            let row_index = row.index();
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

    pub fn player_data_window(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) { // todo: change into tables
        egui::Window::new("Player Data")
            .open(&mut self.show_ui_state.player_data_window)
            .default_width(300.0)
            .vscroll(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|contents| {
                        contents.columns(2, |columns| {
                            columns[0].add(egui::Label::new("Brass"));
                            let brass_response = columns[1].add(egui::DragValue::new(&mut self.player_data.brass));
                            if brass_response.changed() && self.sm.brass_count_node.is_some() {
                                match self.sm.xtree.text_content_mut(self.sm.brass_count_node.unwrap()) {
                                    Some(brass_mut_text) => {
                                        brass_mut_text.set(format!("{}", self.player_data.brass));
                                        if let Some(x) = &mut self.sm.save_tree { // todo: remove invtree, appsaveitem, playerdata coupling
                                            x.update_all_strings(&self.sm.xtree);
                                        };
                                    },
                                    None => {},
                                }
                            };
                        });
                    });

                    ui.add(egui::Label::new("Player stats"));
                    for item in self.player_data.stats.iter_mut() {
                        ui.horizontal(|contents| {
                            contents.columns(2, |columns| {
                                columns[0].add(egui::Label::new(item.1.clone()));
                                let stats_response = columns[1].add(egui::DragValue::new(&mut item.2));
                                if stats_response.changed() {
                                    match self.sm.xtree.text_content_mut(self.sm.stats_nodes[item.0]) {
                                        Some(stat_mut_text) => {
                                            stat_mut_text.set(format!("{}", item.2));
                                            if let Some(x) = &mut self.sm.save_tree { // todo: remove invtree, appsaveitem, playerdata coupling
                                                x.update_all_strings(&self.sm.xtree);
                                            };
                                        },
                                        None => {},
                                    }
                                };
                            });
                        });
                    };

                    ui.horizontal(|contents| {
                        contents.columns(3, |columns| {
                            columns[0].label("Tool");
                            columns[1].label("Level");
                            columns[2].label("XP");
                        });
                    });
                    for item in self.player_data.tool_level.iter_mut() {
                        ui.horizontal(|contents| {
                            contents.columns(3, |columns| {
                                columns[0].add(egui::Label::new(item.1.clone()));
                                let tool_level_response = columns[1].add(egui::DragValue::new(&mut item.2));
                                if tool_level_response.changed() {
                                    match self.sm.xtree.text_content_mut(self.sm.tool_level_ref[item.0].tool_level_node) {
                                        Some(tool_level_mut_text) => {
                                            tool_level_mut_text.set(format!("{}", item.2));
                                            if let Some(x) = &mut self.sm.save_tree { // todo: remove invtree, appsaveitem, playerdata coupling
                                                x.update_all_strings(&self.sm.xtree);
                                            };
                                    },
                                        None => {},
                                    }
                                };
                                let tool_xp_response = columns[2].add(egui::DragValue::new(&mut item.3));
                                if tool_xp_response.changed() {
                                    match self.sm.xtree.text_content_mut(self.sm.tool_level_ref[item.0].tool_current_xp_node) {
                                        Some(tool_xp_mut_text) => {
                                            tool_xp_mut_text.set(format!("{:.1}", item.3));
                                            if let Some(x) = &mut self.sm.save_tree { // todo: remove invtree, appsaveitem, playerdata coupling
                                                x.update_all_strings(&self.sm.xtree);
                                            };
                                        },
                                        None => {},
                                    }
                                };
                            });
                        });
                    };

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
                        Self::reload_data(&mut self.appconfig, &mut self.lm, &mut self.sm, 
                            &mut self.save_inventory_items, &mut self.arm, 
                            &mut self.show_ui_state.error_during_load, &mut self.show_ui_state.error_msg, 
                            &mut self.player_data)

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
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
                if ui.button("Player data").clicked() {self.show_ui_state.player_data_window = !self.show_ui_state.player_data_window;};
                if ui.button("Save tree").clicked() {
                    self.show_ui_state.save_tree_window = !self.show_ui_state.save_tree_window;
                    ui.close_menu();
                };
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
                    body.rows(row_height, num_rows, |mut row| {
                        let row_index = row.index();
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

                                if let Some(x) = &mut self.sm.save_tree { // todo: remove invtree, appsaveitem, playerdata coupling
                                    x.update_all_strings(&self.sm.xtree);
                                };
                            };
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

                                    if let Some(x) = &mut self.sm.save_tree { // todo: remove invtree, appsaveitem, playerdata coupling
                                        x.update_all_strings(&self.sm.xtree);
                                    };
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
                                        if let Some(x) = &mut self.sm.save_tree { // todo: remove invtree, appsaveitem, playerdata coupling
                                            x.reload_data(&self.sm.xtree);
                                        };
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
                                        if let Some(x) = &mut self.sm.save_tree { // todo: remove invtree, appsaveitem, playerdata coupling
                                            x.reload_data(&self.sm.xtree);
                                        };
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

    pub fn update_playerdata(player_data: &mut PlayerData, xtree: &xot::Xot, brass_count_node: &Option<xot::Node>,
        stats_nodes: &Vec<xot::Node>, tool_level_ref: &Vec<savedata::ToolLevelRef>) 
    {
        player_data.brass = {
            match xtree.text_content_str(brass_count_node.unwrap()) {
                Some(x) => x.parse().unwrap_or_else(|_| {0}),
                _ => {0}
            }
        };

        for (idx, item) in stats_nodes.iter().enumerate() {
            let stat_val_str = xtree.text_content_str(*item);
            for stat_item in player_data.stats.iter_mut() {
                if stat_val_str.is_some() && stat_item.0 == idx {
                    let stat_val: u8 = stat_val_str.unwrap().parse().unwrap_or_else(|_| {0});
                    stat_item.2 = stat_val;
                }
            }
        };

        for (idx, item) in tool_level_ref.iter().enumerate() {
            let tool_level_val_str = xtree.text_content_str(item.tool_level_node);
            let tool_xp_val_str = xtree.text_content_str(item.tool_current_xp_node);
            for tool_level_item in player_data.tool_level.iter_mut() {
                if tool_level_val_str.is_some() && tool_xp_val_str.is_some() && tool_level_item.0 == idx {
                    let tool_level_val: u8 = tool_level_val_str.unwrap().parse().unwrap_or_else(|_| {0});
                    let tool_xp_val: f32 = tool_xp_val_str.unwrap().parse().unwrap_or_else(|_| {0.0});
                    tool_level_item.2 = tool_level_val;
                    tool_level_item.3 = tool_xp_val;
                }
            }
        };
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
            player_data: _,
        } = self;
        
        if self.show_ui_state.top_panel {self.top_panel(ctx, frame)};
        if self.show_ui_state.top_panel {self.bottom_panel(ctx, frame)};
        if self.show_ui_state.central_panel {self.central_panel(ctx, frame)};
        if self.show_ui_state.options_window {self.options_window(ctx, frame)};
        if self.show_ui_state.loot_ref_window {self.loot_ref_window(ctx, frame)};
        if self.show_ui_state.player_data_window {self.player_data_window(ctx, frame)};
        if self.show_ui_state.save_tree_window {self.save_tree_window(ctx, frame)};

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

    pub fn new_xt(save_item_ref: &savedata::SaveInventoryItemRef, xtree: &xot::Xot, lm: &lootitems::LootManager) -> Self {
        let sir = save_item_ref.clone();
        let uid = sir.get_uid_xt(xtree);
        let counts = sir.get_counts_xt(xtree).map(SaveInventoryItemCount::new);
        let li = sir.get_lootitem_ref_xt(xtree,lm);
        let li_name = li.name.to_string();
        let li_pickup_type = lm.pickup_type_lookup_rev[&li.type_of_pickup].to_string();
        let li_cost = li.cost;

        Self {
            save_item_ref: sir,
            uid,
            counts,
            name: li_name,
            pickup_type_name: li_pickup_type,
            cost: li_cost,
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

    pub fn update_fromref_xt(&mut self, xtree: &xot::Xot, lm: &lootitems::LootManager) {
        let sir = &self.save_item_ref;
        self.uid = sir.get_uid_xt(xtree);
        self.counts = sir.get_counts_xt(xtree).map(SaveInventoryItemCount::new);
        let li = sir.get_lootitem_ref_xt(xtree,lm);
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
    check_dupe_uids(sm)?;
    backup_save(appconfig).unwrap();

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