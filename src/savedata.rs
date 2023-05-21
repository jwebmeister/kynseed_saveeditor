use std::path::PathBuf;
use std::error::Error;

use crate::config::AppConfig;
use crate::lootitems::{LootManager, LootItem};

#[derive(Debug, Clone)]
struct SaveDataError;

impl std::fmt::Display for SaveDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to load save data")
    }
}

impl Error for SaveDataError {}


#[derive(Debug, Clone, PartialEq)]
pub struct SaveInventoryItemRef {
    pub item_node: xot::Node,
    pub key_int_node: xot::Node,
    pub count_int_nodes: Vec<xot::Node>
}

impl SaveInventoryItemRef {

    pub fn get_uid(&self, sm: &SaveDataManager) -> i32 {
        let uid = sm.xtree.text_content_str(self.key_int_node).unwrap().parse::<i32>().unwrap();
        uid
    }

    pub fn set_uid(&self, sm: &mut SaveDataManager, new_uid: i32) -> i32 {
        let uid_text = sm.xtree.text_content_mut(self.key_int_node).unwrap();
        uid_text.set(new_uid.to_string());
        let uid_as_set = uid_text.get().parse::<i32>().unwrap();
        uid_as_set
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

    pub fn remove(&mut self, sm: &mut SaveDataManager, lir: LocationItemRef) -> Result<(), Box<dyn Error>> {
        let nodeclone = self.clone();
        match sm.xtree.remove(self.item_node) {
            Ok(_) => {
                match lir {
                    LocationItemRef::Inventory => {sm.save_inventory_ref.retain(|x| x.item_node != nodeclone.item_node)},
                    LocationItemRef::NewLarder => {sm.newlarder_item_ref.retain(|x| x.item_node != nodeclone.item_node)},
                    LocationItemRef::SavedShops => {sm.savedshops_item_ref.retain(|x| x.item_node != nodeclone.item_node)}
                };
                Ok(())
            },
            Err(e) => Err(Box::new(e))
        }
    }

    pub fn copy_new(&mut self, sm: &mut SaveDataManager, lir: LocationItemRef) -> Result<SaveInventoryItemRef, Box<dyn Error>> {
        let nodeclone = sm.xtree.clone(self.item_node);
        match sm.xtree.insert_after(self.item_node, nodeclone) {
            Ok(_) => {
                match lir {
                    LocationItemRef::Inventory => {
                        let child = nodeclone;
                        let child_key_node = sm.get_child_node_from_name( child, "key")
                            .ok_or_else(|| Box::new(SaveDataError))?;
                        let child_key_int_node = sm.get_child_node_from_name( child_key_node, "int")
                            .ok_or_else(|| Box::new(SaveDataError))?;

                        let child_value_node = sm.get_child_node_from_name( child, "value")
                            .ok_or_else(|| Box::new(SaveDataError))?;
                        let child_value_inventoryitem_node = sm.get_child_node_from_name( child_value_node, "InventoryItem")
                            .ok_or_else(|| Box::new(SaveDataError))?;
                        let child_value_inventoryitem_count_node = sm.get_child_node_from_name( child_value_inventoryitem_node, "Count")
                            .ok_or_else(|| Box::new(SaveDataError))?;
                        
                        let mut child_count_int_nodes: Vec<xot::Node> = Vec::new();
                        
                        for maybe_int_node in sm.xtree.children(child_value_inventoryitem_count_node) {
                            match sm.get_name_from_node( maybe_int_node) {
                                None => continue,
                                Some(el_name) => {
                                    if el_name == "int" {
                                        child_count_int_nodes.push(maybe_int_node);
                                        
                                    }
                                }
                            }
                        };

                        let save_inventory_item_ref = SaveInventoryItemRef{item_node: child, key_int_node: child_key_int_node, count_int_nodes: child_count_int_nodes };
                        let siir_clone = save_inventory_item_ref.clone();
                        sm.save_inventory_ref.push(save_inventory_item_ref);
                        Ok(siir_clone)
                    },
                    LocationItemRef::NewLarder => todo!(),
                    LocationItemRef::SavedShops => todo!(),
                }
            },
            Err(e) => Err(Box::new(e))
        }
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

pub enum LocationItemRef {
    Inventory,
    NewLarder,
    SavedShops
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

    pub fn clear_data(&mut self) {
        self.root = None;
        self.doc_el = None;
        self.playerdata_node = None;
        self.brass_count_node = None;
        self.tool_levelling_node = None;
        self.inventory_node = None;
        self.allitems_node = None;
        self.save_inventory_ref.clear();
        self.newlarder_node = None;
        self.newlarder_item_ref.clear();
        self.savedshops_node = None;
        self.savedshops_item_ref.clear();
        self.xtree = xot::Xot::new();
    }

    pub fn load_data(&mut self, appconfig: &AppConfig) -> Result<(), Box<dyn Error>> {

        let filepath_savegame = PathBuf::from_iter([&appconfig.path_kynseed_saves, &appconfig.filename_kynseed_save]);

        let mut xml_vec = std::fs::read(&filepath_savegame)?;

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

        let xml_str = std::str::from_utf8(&xml_vec)?;

        // should fix in xot crate
        // self.root = Some(self.xtree.parse(&xml_str.replacen("utf-8", "UTF-8", 1)).unwrap());

        self.root = Some(self.xtree.parse(xml_str)?);
        self.doc_el = match self.xtree.document_element(self.root.unwrap()) {
            Ok(x) => Some(x),
            Err(e) => return Err(Box::new(e))
        };

        // println!("{:?}", self.get_name_from_node(self.doc_el.unwrap()));

        self.playerdata_node = match self.get_child_node_from_name(self.doc_el.unwrap(), "PlayerData") {
            Some(x) => Some(x),
            None => return Err(Box::new(SaveDataError))
        };

        self.brass_count_node = match self.get_child_node_from_name(self.playerdata_node.unwrap(), "BrassCount") {
            Some(x) => Some(x),
            None => return Err(Box::new(SaveDataError))
        };
        self.tool_levelling_node = match self.get_child_node_from_name(self.playerdata_node.unwrap(), "ToolLevelling") {
            Some(x) => Some(x),
            None => return Err(Box::new(SaveDataError))
        };

        self.inventory_node = match self.get_child_node_from_name( self.playerdata_node.unwrap(), "Inventory") {
            Some(x) => Some(x),
            None => return Err(Box::new(SaveDataError))
        };
        self.allitems_node = match self.get_child_node_from_name(self.inventory_node.unwrap(), "AllItems") {
            Some(x) => Some(x),
            None => return Err(Box::new(SaveDataError))
        };

        for child in self.xtree.children(self.allitems_node.unwrap()) {
            match self.get_name_from_node(child) {
                None => continue,
                Some(child_el_name) => if child_el_name != "item" {continue}
            };
            
            let child_key_node = self.get_child_node_from_name( child, "key")
                .ok_or_else(|| Box::new(SaveDataError))?;
            let child_key_int_node = self.get_child_node_from_name( child_key_node, "int")
                .ok_or_else(|| Box::new(SaveDataError))?;

            let child_value_node = self.get_child_node_from_name( child, "value")
                .ok_or_else(|| Box::new(SaveDataError))?;
            let child_value_inventoryitem_node = self.get_child_node_from_name( child_value_node, "InventoryItem")
                .ok_or_else(|| Box::new(SaveDataError))?;
            let child_value_inventoryitem_count_node = self.get_child_node_from_name( child_value_inventoryitem_node, "Count")
                .ok_or_else(|| Box::new(SaveDataError))?;
            
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

            let save_inventory_item_ref = SaveInventoryItemRef{item_node: child, key_int_node: child_key_int_node, count_int_nodes: child_count_int_nodes };
            self.save_inventory_ref.push(save_inventory_item_ref);

        };

        self.load_newlarder_data()?;
        self.load_savedshops_data()?;

        Ok(())
        
    }

    pub fn load_newlarder_data(&mut self) -> Result<(), Box<dyn Error>> {
        self.newlarder_node = self.get_child_node_from_name(self.playerdata_node.unwrap(), "newLarder");
        match self.newlarder_node {
            None => return Ok(()),
            Some(_node) => {}
        };

        for descendant in self.xtree.descendants(self.newlarder_node.unwrap()) {
            match self.get_name_from_node(descendant) {
                None => continue,
                Some(d_name) => {
                    if d_name == "ItemStack" {
                        let d_key_node = self.get_child_node_from_name(descendant, "UniqueID")
                            .ok_or_else(|| Box::new(SaveDataError))?;
                        let mut d_count_int_nodes:Vec<xot::Node> = Vec::new();
                        if let Some(d_count_node) = self.get_child_node_from_name(descendant, "Count") {
                            for maybe_int_node in self.xtree.children(d_count_node) {
                                match self.get_name_from_node( maybe_int_node) {
                                    None => continue,
                                    Some(el_name) => {if el_name == "int" {d_count_int_nodes.push(maybe_int_node);}}
                                }
                            };
                        };
                        self.newlarder_item_ref.push(SaveInventoryItemRef{item_node: descendant, key_int_node: d_key_node, count_int_nodes: d_count_int_nodes });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn load_savedshops_data(&mut self) -> Result<(), Box<dyn Error>>{
        self.savedshops_node = self.get_child_node_from_name(self.doc_el.unwrap(), "SavedShops");
        match self.savedshops_node {
            None => return Ok(()),
            Some(_node) => {}
        };

        for descendant in self.xtree.descendants(self.savedshops_node.unwrap()) {
            match self.get_name_from_node(descendant) {
                None => continue,
                Some(d_name) => {
                    if d_name == "ItemStack" {
                        let d_key_node = self.get_child_node_from_name(descendant, "UniqueID")
                            .ok_or_else(|| Box::new(SaveDataError))?;
                        let mut d_count_int_nodes:Vec<xot::Node> = Vec::new();
                        if let Some(d_count_node) = self.get_child_node_from_name(descendant, "Count") {
                            for maybe_int_node in self.xtree.children(d_count_node) {
                                match self.get_name_from_node( maybe_int_node) {
                                    None => continue,
                                    Some(el_name) => {if el_name == "int" {d_count_int_nodes.push(maybe_int_node);}}
                                }
                            };
                        };
                        self.savedshops_item_ref.push(SaveInventoryItemRef{item_node: descendant, key_int_node: d_key_node, count_int_nodes: d_count_int_nodes });
                    }
                }
            }
        };
        Ok(())

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