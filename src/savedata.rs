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

    pub fn get_uid_xt(&self, xtree: &xot::Xot) -> i32 {
        let uid = xtree.text_content_str(self.key_int_node).unwrap().parse::<i32>().unwrap();
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

    pub fn get_lootitem_ref_xt<'a>(&'a self, xtree: &xot::Xot, lm: &'a LootManager) -> &LootItem {
        let uid = self.get_uid_xt(xtree);
        
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

    pub fn get_counts_xt(&self, xtree: &xot::Xot) -> [i32; 5] {
        let mut counts: [i32; 5] = [0; 5];
        for (idx, count_ref) in self.count_int_nodes.iter().enumerate() {
            let count_text = xtree.text_content_str(*count_ref).unwrap();
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


pub struct ToolLevelRef {
    pub tool_node: xot::Node,
    pub tool_type_node: xot::Node,
    pub tool_level_node: xot::Node,
    pub tool_current_xp_node: xot::Node,
}

pub struct SaveDataManager {
    pub xtree: xot::Xot,
    pub root: Option<xot::Node>,
    pub doc_el: Option<xot::Node>,
    pub playerdata_node: Option<xot::Node>,
    pub brass_count_node: Option<xot::Node>,
    pub character_stats_node: Option<xot::Node>,
    pub stats_nodes: Vec<xot::Node>,
    pub tool_levelling_node: Option<xot::Node>,
    pub tool_level_ref: Vec<ToolLevelRef>,
    pub inventory_node: Option<xot::Node>,
    pub allitems_node: Option<xot::Node>,
    pub save_inventory_ref: Vec<SaveInventoryItemRef>,
    pub newlarder_node: Option<xot::Node>,
    pub newlarder_item_ref: Vec<SaveInventoryItemRef>,
    pub savedshops_node: Option<xot::Node>,
    pub savedshops_item_ref: Vec<SaveInventoryItemRef>,

    pub save_tree: Option<SaveNodeTree>,
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
            character_stats_node: None,
            stats_nodes: Vec::new(),
            tool_levelling_node: None,
            tool_level_ref: Vec::new(),
            inventory_node: None,
            allitems_node: None,
            save_inventory_ref: Vec::new(),
            newlarder_node: None,
            newlarder_item_ref: Vec::new(),
            savedshops_node: None,
            savedshops_item_ref: Vec::new(),

            save_tree: None,
        }
    }
}

impl SaveDataManager {

    pub fn clear_data(&mut self) {
        self.root = None;
        self.doc_el = None;
        self.playerdata_node = None;
        self.brass_count_node = None;
        self.character_stats_node = None;
        self.stats_nodes.clear();
        self.tool_levelling_node = None;
        self.tool_level_ref.clear();
        self.inventory_node = None;
        self.allitems_node = None;
        self.save_inventory_ref.clear();
        self.newlarder_node = None;
        self.newlarder_item_ref.clear();
        self.savedshops_node = None;
        self.savedshops_item_ref.clear();
        self.xtree = xot::Xot::new();

        self.save_tree = None;
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
        self.character_stats_node = match self.get_child_node_from_name(self.playerdata_node.unwrap(), "characterStats") {
            Some(x) => Some(x),
            None => return Err(Box::new(SaveDataError))
        };
        for c_stats_node in self.xtree.children(self.character_stats_node.unwrap()) {
            match self.get_name_from_node(c_stats_node) {
                None => continue,
                Some(child_el_name) => if !child_el_name.contains("BASE_") {continue}
            };
            self.stats_nodes.push(c_stats_node);
        }

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

        self.load_tool_levels()?;

        self.save_tree = Some(SaveNodeTree::new(&self.doc_el.unwrap(), &self.xtree));

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

    pub fn load_tool_levels(&mut self) -> Result<(), Box<dyn Error>>{
        match self.tool_levelling_node {
            None => return Ok(()),
            Some(_node) => {}
        };

        for child in self.xtree.children(self.tool_levelling_node.unwrap()) {
            match self.get_name_from_node(child) {
                None => continue,
                Some(c_name) => {
                    if c_name == "ToolLevel" {
                        let c_type_node = self.get_child_node_from_name(child, "type")
                            .ok_or_else(|| Box::new(SaveDataError))?;
                        let c_level_node = self.get_child_node_from_name(child, "Level")
                            .ok_or_else(|| Box::new(SaveDataError))?;
                        let c_currentxp_node = self.get_child_node_from_name(child, "ExactCurrentXP")
                            .ok_or_else(|| Box::new(SaveDataError))?;
                        self.tool_level_ref.push(ToolLevelRef { tool_node: child, tool_type_node: c_type_node, tool_level_node: c_level_node, tool_current_xp_node: c_currentxp_node });
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

    pub fn get_name_from_node_xt(xtree: &xot::Xot, node: xot::Node) -> Option<&str> {
        let node_el_result = xtree.element(node);
        let node_el_name: &str;
        match node_el_result {
            None => None,
            Some(node_el) => {
                (node_el_name, _) = xtree.name_ns_str(node_el.name());
                Some(node_el_name)
            }
        }
    }

    pub fn get_child_node_from_name_xt(xtree: &xot::Xot, parent_node: xot::Node, name: &str) -> Option<xot::Node> {
        for child in xtree.children(parent_node) {
            let child_el_name_result = Self::get_name_from_node_xt(xtree, child);
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

    pub fn get_sir_from_item_node(child: xot::Node, xtree: &xot::Xot) -> Result<SaveInventoryItemRef, Box<dyn Error>> {
        let child_key_node = Self::get_child_node_from_name_xt( xtree, child, "key")
            .ok_or_else(|| Box::new(SaveDataError))?;
        let child_key_int_node = Self::get_child_node_from_name_xt( xtree, child_key_node, "int")
            .ok_or_else(|| Box::new(SaveDataError))?;

        let child_value_node = Self::get_child_node_from_name_xt( xtree, child, "value")
            .ok_or_else(|| Box::new(SaveDataError))?;
        let child_value_inventoryitem_node = Self::get_child_node_from_name_xt( xtree, child_value_node, "InventoryItem")
            .ok_or_else(|| Box::new(SaveDataError))?;
        let child_value_inventoryitem_count_node = Self::get_child_node_from_name_xt( xtree, child_value_inventoryitem_node, "Count")
            .ok_or_else(|| Box::new(SaveDataError))?;
        
        let mut child_count_int_nodes: Vec<xot::Node> = Vec::new();
        
        for maybe_int_node in xtree.children(child_value_inventoryitem_count_node) {
            match Self::get_name_from_node_xt( xtree, maybe_int_node) {
                None => continue,
                Some(el_name) => {
                    if el_name == "int" {
                        child_count_int_nodes.push(maybe_int_node);
                        
                    }
                }
            }
        };

        let save_inventory_item_ref = SaveInventoryItemRef{item_node: child, key_int_node: child_key_int_node, count_int_nodes: child_count_int_nodes };
        Ok(save_inventory_item_ref)
    }
}

// SaveNodeTree(Node, Name, Text Content, b Has Text Content, Children).
// xtree is xot::Xot which every Node references
#[derive(Clone)]
pub struct SaveNodeTree(pub xot::Node, pub String, pub String, pub bool, pub Vec<SaveNodeTree>);

impl SaveNodeTree {

    pub fn new(node: &xot::Node, xtree: &xot::Xot) -> Self {
        let name: String;
        let text: String;
        match Self::get_name_from_node(node, xtree) {
            Some(s) => {name = s.to_string();},
            None => {name = String::default();}
        };
        match Self::get_str_from_node(node, xtree) {
            Some(t) => {text = t.to_string();},
            None => {text = String::default();}
        };
        let b_has_text_content = !text.trim().is_empty();
        let children = Self::get_good_children_from_node(node, xtree);
        Self {
            0: node.clone(),
            1: name,
            2: text,
            3: b_has_text_content,
            4: children,
        }
    }

    pub fn reload_data(&mut self, xtree: &xot::Xot) {
        let name: String;
        let text: String;
        match Self::get_name_from_node(&self.0, xtree) {
            Some(s) => {name = s.to_string();},
            None => {name = String::default();}
        };
        match Self::get_str_from_node(&self.0, xtree) {
            Some(t) => {text = t.to_string();},
            None => {text = String::default();}
        };
        let b_has_text_content = !text.trim().is_empty();
        let children = Self::get_good_children_from_node(&self.0, xtree);
        self.1 = name;
        self.2 = text;
        self.3 = b_has_text_content;
        self.4 = children;
    }

    pub fn get_good_children_from_node(node: &xot::Node, xtree: &xot::Xot) -> Vec<Self> {
        let mut good_children: Vec<Self> = vec![];
        for child in xtree.children(node.clone()) {
            let mut b_good_child = false;
            if let Some(c_name) = Self::get_name_from_node(&child, xtree) {
                if !c_name.trim().is_empty() {b_good_child = true;}
            };
            if let Some(c_text) = Self::get_str_from_node(&child, xtree) {
                if !c_text.trim().is_empty() {b_good_child = true;}
            };
            if b_good_child {
                good_children.push(Self::new(&child, xtree));
            };
        };
        good_children
    }

    pub fn get_name_from_node<'a>(node: &'a xot::Node, xtree: &'a xot::Xot) -> Option<&'a str> {
        let node_el_option = xtree.element(*node);
        match node_el_option {
            None => None,
            Some(node_el) => {
                let (node_el_name, _) = xtree.name_ns_str(node_el.name());
                Some(node_el_name)
            }
        }
    }

    pub fn get_text_content_from_node<'a>(node: &'a xot::Node, xtree: &'a xot::Xot) -> Option<&'a xot::Text> {
        xtree.text_content(*node)
    }

    pub fn get_text_content_mut_from_node<'a>(node: &'a xot::Node, xtree: &'a mut xot::Xot) -> Option<&'a mut xot::Text> {
        xtree.text_content_mut(*node)
    }

    pub fn get_str_from_node<'a>(node: &'a xot::Node, xtree: &'a xot::Xot) -> Option<&'a str> {
        let node_text_option = xtree.text_content(*node);
        match node_text_option {
            None => None,
            Some(node_text) => {
                Some(node_text.get())
            }
        }
    }

    pub fn set_str_from_node<'a, S: Into<String>>(node: &'a xot::Node, xtree: &'a mut xot::Xot, text: S) -> Option<&'a str> {
        let node_text_option = xtree.text_content_mut(*node);
        match node_text_option {
            None => None,
            Some(node_text) => {
                node_text.set(text);
                Some(node_text.get().clone())
            }
        }
    }

    pub fn update_strings_from_self(&mut self, xtree: &xot::Xot) {
        if let Some(name) = Self::get_name_from_node(&self.0, xtree) {
            self.1 = name.to_string();
        } else {
            self.1 = String::default();
        };

        if let Some(text_content) = Self::get_str_from_node(&self.0, xtree) {
            self.2 = text_content.to_string();
        } else {
            self.2 = String::default();
        };
    }

    pub fn update_all_strings(&mut self, xtree: &xot::Xot) {
        self.update_strings_from_self(xtree);
        self.4.iter_mut().for_each(|child| child.update_all_strings(xtree));
    }

    pub fn set_str_from_self(&mut self, xtree: &mut xot::Xot) {
        let return_str = Self::set_str_from_node(&self.0, xtree, self.2.clone());
        if let Some(x) = return_str {self.2 = x.to_string()} else {self.2 = String::default()};
    }

    pub fn set_str_from_self_with_check<F>(&mut self, xtree: &mut xot::Xot, check_func: F) 
        where F: FnOnce(String) -> Option<String>
    {
        let desired_val = self.2.clone();
        let return_str: Option<&str>;

        if let Some(s) = check_func(desired_val) {
            return_str = Self::set_str_from_node(&self.0, xtree, s);
        } else {
            return_str = Self::get_str_from_node(&self.0, xtree);
        };

        if let Some(x) = return_str {
            self.2 = x.to_string()
        } else {
            self.2 = String::default()
        };
    }

    pub fn find_parent_childidx_from_node<'a>(root: &'a mut Self, node: &'a xot::Node) -> Result<(&'a mut Self, usize), String> {
        let mut child_idx: Option<usize> = None;

        if root.0 == *node {return Err("No parent, node is root.".to_string())};
        if root.4.is_empty() {return Err("No children.".to_string())};

        for (idx, child) in root.4.iter().enumerate() {
            if child.0 == *node {
                child_idx = Some(idx); 
                break;
            };
        };
        if child_idx.is_some() {
            return Ok((root, child_idx.unwrap()));
        };

        for child in root.4.iter_mut() {
            if let Ok(child_result) = Self::find_parent_childidx_from_node(child, node) {
                return Ok(child_result);
            } else {
                continue;
            }
        }
        Err("Could not find node in tree.".to_string())
    }

    pub fn copy_node(root: &mut Self, xtree: &mut xot::Xot, node: &xot::Node) -> Result<xot::Node, String> {
        let node_deref = node.clone();
        let new_node = xtree.clone(node_deref);
        if let Ok((parent, child_idx)) = Self::find_parent_childidx_from_node(root, &node) {
            match xtree.insert_after(node_deref, new_node) {
                Ok(_) => {
                    let new_tree = Self::new(&new_node, xtree);
                    parent.4.insert(child_idx, new_tree);
                    return Ok(new_node);
                },
                Err(_) => {
                    if let Err(_e) = xtree.remove(new_node) {
                        return Err("Could not add node to tree. Could not remove new unattached node, uh oh!".to_string());
                    };
                    return Err("Could not add node to tree.".to_string());
                }
            }
        } else {
            if let Err(_e) = xtree.remove(new_node) {
                return Err("Could not find node in tree. Could not remove new unattached node, uh oh!".to_string());
            }
            return Err("Could not find node in tree.".to_string());
        }
    }

    pub fn remove_node(root: &mut Self, xtree: &mut xot::Xot, node: &xot::Node) -> Result<(), String> {
        if let Ok((parent, child_idx)) = Self::find_parent_childidx_from_node(root, node) {
            let node_deref = node.clone();
            match xtree.remove(node_deref) {
                Ok(_) => {
                    parent.4.remove(child_idx);
                    return Ok(());
                },
                Err(_) => {
                    return Err("Could not remove node.".to_string());
                }
            }
        } else {
            return Err("Could not find node in tree.".to_string());
        }
    }

}