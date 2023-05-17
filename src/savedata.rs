use std::path::PathBuf;

use crate::config::AppConfig;
use crate::lootitems::{LootManager, LootItem};



#[derive(Debug, Clone)]
pub struct SaveInventoryItemRef {
    pub key_int_node: xot::Node,
    pub count_int_nodes: Vec<xot::Node>
}

impl SaveInventoryItemRef {

    pub fn get_uid(&self, sm: &SaveDataManager) -> i32 {
        let uid = sm.xtree.text_content_str(self.key_int_node).unwrap().parse::<i32>().unwrap();
        uid
    }

    pub fn get_lootitem_ref<'a>(&'a self, sm: &SaveDataManager, lm: &'a LootManager) -> &LootItem {
        let uid = self.get_uid(sm);
        
        &lm.full_item_lookup[&uid] as _
    }

    pub fn get_counts(&self, sm: &SaveDataManager) -> [i32; 5] {
        let mut counts: [i32; 5] = [0; 5];
        for (idx, count_ref) in self.count_int_nodes.iter().enumerate() {
            let count_text = sm.xtree.text_content_str(*count_ref).unwrap();
            let count_int = count_text.parse::<i32>().unwrap();
            counts[idx] = count_int;
        };
        counts
    }

    // pub fn get_count_at_idx(&self, idx: usize, sm: &SaveDataManager) -> i32 {
    //     let count_ref = self.count_int_nodes[idx];
    //     let count_text = sm.xtree.text_content_str(count_ref).unwrap();
    //     let count_int = count_text.parse::<i32>().unwrap();
    //     return count_int;
    // }

    pub fn set_count_at_idx(&self, idx: usize, new_count: i32, sm: &mut SaveDataManager, lm: Option<&LootManager>) -> i32 {
        let uid = self.get_uid(sm);
        let mut max_qty = 999;
        match lm {
            None => {},
            Some(lmgr) => {
                max_qty = lmgr.get_max_item_quantity(uid)[idx];
            }
        };
        let clamped_new_count = new_count.clamp(0, max_qty);
        let count_ref = self.count_int_nodes[idx];
        let count_text = sm.xtree.text_content_mut(count_ref).unwrap();
        count_text.set(clamped_new_count.to_string());
        let count_as_set = count_text.get().parse::<i32>().unwrap();
        count_as_set
    }


}

pub struct SaveDataManager {
    pub xtree: xot::Xot,
    pub root: Option<xot::Node>,
    pub doc_el: Option<xot::Node>,
    pub playerdata_node: Option<xot::Node>,
    pub brass_count_node: Option<xot::Node>,
    pub tool_levelling_node: Option<xot::Node>,
    pub inventory_node: Option<xot::Node>,
    pub allitems_node: Option<xot::Node>,
    pub save_inventory_ref: Vec<SaveInventoryItemRef>,
    pub newlarder_node: Option<xot::Node>,
    pub newlarder_item_ref: Vec<SaveInventoryItemRef>,
    pub savedshops_node: Option<xot::Node>,
    pub savedshops_item_ref: Vec<SaveInventoryItemRef>,
}

impl Default for SaveDataManager {
    fn default() -> Self {
        Self {
            xtree: xot::Xot::new(),
            root: None,
            doc_el: None,
            playerdata_node: None,
            brass_count_node: None,
            tool_levelling_node: None,
            inventory_node: None,
            allitems_node: None,
            save_inventory_ref: Vec::new(),
            newlarder_node: None,
            newlarder_item_ref: Vec::new(),
            savedshops_node: None,
            savedshops_item_ref: Vec::new(),
        }
    }
}

impl SaveDataManager {

    pub fn load_data(&mut self, appconfig: &AppConfig) {

        let filepath_savegame = PathBuf::from_iter([&appconfig.path_kynseed_saves, &appconfig.filename_kynseed_save]);

        let mut xml_vec = std::fs::read(&filepath_savegame).unwrap();

        // remove BOM
        if xml_vec[0..3] == [b'\xef', b'\xbb', b'\xbf'] {
            xml_vec.remove(0);
            xml_vec.remove(0);
            xml_vec.remove(0);
        };
        // should fix in xot crate
        for i in 0..50 {
            // dbg!(xml_vec[i], xml_vec[i+1], xml_vec[i+2], xml_vec[i+3], xml_vec[i+4]);
            if xml_vec[i] as char == 'u' 
                && xml_vec[i+1] as char == 't' 
                && xml_vec[i+2] as char == 'f' 
                && xml_vec[i+3] as char == '-' 
                && xml_vec[i+4] as char == '8' 
                {
                    xml_vec[i] = b'U';
                    xml_vec[i+1] = b'T';
                    xml_vec[i+2] = b'F';
                    break;
            };
        };

        let xml_str = std::str::from_utf8(&xml_vec).unwrap();

        // should fix in xot crate
        // self.root = Some(self.xtree.parse(&xml_str.replacen("utf-8", "UTF-8", 1)).unwrap());

        self.root = Some(self.xtree.parse(xml_str).unwrap());
        self.doc_el = Some(self.xtree.document_element(self.root.unwrap()).unwrap());

        // println!("{:?}", self.get_name_from_node(self.doc_el.unwrap()));

        self.playerdata_node = Some(self.get_child_node_from_name(self.doc_el.unwrap(), "PlayerData").unwrap());

        self.brass_count_node = Some(self.get_child_node_from_name(self.playerdata_node.unwrap(), "BrassCount").unwrap());
        self.tool_levelling_node = Some(self.get_child_node_from_name(self.playerdata_node.unwrap(), "ToolLevelling").unwrap());

        self.inventory_node = Some(self.get_child_node_from_name( self.playerdata_node.unwrap(), "Inventory").unwrap());
        self.allitems_node = Some(self.get_child_node_from_name(self.inventory_node.unwrap(), "AllItems").unwrap());

        for child in self.xtree.children(self.allitems_node.unwrap()) {
            match self.get_name_from_node( child) {
                None => continue,
                Some(child_el_name) => if child_el_name != "item" {continue}
            };
            
            let child_key_node = self.get_child_node_from_name( child, "key").unwrap();
            let child_key_int_node = self.get_child_node_from_name( child_key_node, "int").unwrap();

            let child_value_node = self.get_child_node_from_name( child, "value").unwrap();
            let child_value_inventoryitem_node = self.get_child_node_from_name( child_value_node, "InventoryItem").unwrap();
            let child_value_inventoryitem_count_node = self.get_child_node_from_name( child_value_inventoryitem_node, "Count").unwrap();
            
            let mut child_count_int_nodes: Vec<xot::Node> = Vec::new();
            
            for maybe_int_node in self.xtree.children(child_value_inventoryitem_count_node) {
                match self.get_name_from_node( maybe_int_node) {
                    None => continue,
                    Some(el_name) => {
                        if el_name == "int" {
                            child_count_int_nodes.push(maybe_int_node);
                            
                        }
                    }
                }
            };

            let save_inventory_item_ref = SaveInventoryItemRef{key_int_node: child_key_int_node, count_int_nodes: child_count_int_nodes };
            self.save_inventory_ref.push(save_inventory_item_ref);

        };

        self.load_newlarder_data();
        self.load_savedshops_data();
        
    }

    pub fn load_newlarder_data(&mut self) {
        self.newlarder_node = self.get_child_node_from_name(self.playerdata_node.unwrap(), "newLarder");
        match self.newlarder_node {
            None => return,
            Some(_node) => {}
        };

        for descendant in self.xtree.descendants(self.newlarder_node.unwrap()) {
            match self.get_name_from_node(descendant) {
                None => continue,
                Some(d_name) => {
                    if d_name == "ItemStack" {
                        let d_key_node = self.get_child_node_from_name(descendant, "UniqueID").unwrap();
                        let d_count_node = self.get_child_node_from_name(descendant, "Count");
                        let mut d_count_int_nodes:Vec<xot::Node> = Vec::new();

                        for maybe_int_node in self.xtree.children(d_count_node.unwrap()) {
                            match self.get_name_from_node( maybe_int_node) {
                                None => continue,
                                Some(el_name) => {
                                    if el_name == "int" {
                                        d_count_int_nodes.push(maybe_int_node);
                                        
                                    }
                                }
                            }
                        };

                        self.newlarder_item_ref.push(SaveInventoryItemRef{key_int_node: d_key_node, count_int_nodes: d_count_int_nodes });
                    }
                }

            }
        }

    }

    pub fn load_savedshops_data(&mut self) {
        self.savedshops_node = self.get_child_node_from_name(self.doc_el.unwrap(), "SavedShops");
        match self.savedshops_node {
            None => return,
            Some(_node) => {}
        };

        for descendant in self.xtree.descendants(self.savedshops_node.unwrap()) {
            match self.get_name_from_node(descendant) {
                None => continue,
                Some(d_name) => {
                    if d_name == "ItemStack" {
                        let d_key_node = self.get_child_node_from_name(descendant, "UniqueID").unwrap();
                        let d_count_node = self.get_child_node_from_name(descendant, "Count");
                        let mut d_count_int_nodes:Vec<xot::Node> = Vec::new();

                        for maybe_int_node in self.xtree.children(d_count_node.unwrap()) {
                            match self.get_name_from_node( maybe_int_node) {
                                None => continue,
                                Some(el_name) => {
                                    if el_name == "int" {
                                        d_count_int_nodes.push(maybe_int_node);
                                        
                                    }
                                }
                            }
                        };

                        self.savedshops_item_ref.push(SaveInventoryItemRef{key_int_node: d_key_node, count_int_nodes: d_count_int_nodes });
                    }
                }

            }
        }

    }

    pub fn get_name_from_node(&self, node: xot::Node) -> Option<&str> {
        let node_el_result = self.xtree.element(node);
        let node_el_name: &str;
        match node_el_result {
            None => None,
            Some(node_el) => {
                (node_el_name, _) = self.xtree.name_ns_str(node_el.name());
                Some(node_el_name)
            }
        }
    }

    pub fn get_child_node_from_name(&self, parent_node: xot::Node, name: &str) -> Option<xot::Node> {
        for child in self.xtree.children(parent_node) {
            let child_el_name_result = self.get_name_from_node(child);
            // println!("{:?}", child_el_name_result);
            match child_el_name_result {
                None => continue,
                Some(child_el_name) => {
                    if child_el_name == name {
                        let child_node = child;
                        // println!("{:?}", child_el_name);
                        return Some(child_node);
                    }
                }
            }
        };
        None
    }
}