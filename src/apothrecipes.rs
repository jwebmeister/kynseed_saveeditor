use std::path::PathBuf;

use std::str::FromStr;
use strum::EnumString;

use crate::config::AppConfig;



#[derive(Debug, Clone, EnumString, PartialEq)]
pub enum CureResultType {
    BrandedCure = 0,
    OffBrandCure = 1,
    PartialCure = 2,
    FailureCure = 3,
    Placebo = 4,
}

#[derive(Debug, Clone)]
pub struct ApothRecipe {
    result_type: CureResultType,
    ailment_id: i32,
    item_id: i32,
    item_id_with_side_effects: i32,
}

pub struct ApothRecipeManager {
    pub xtree: xot::Xot,
    pub root: Option<xot::Node>,
    pub doc_el: Option<xot::Node>,
    pub apothrecipes_node: Option<xot::Node>,
    pub all_cures: Vec<ApothRecipe>,
}

impl Default for ApothRecipeManager {
    fn default() -> Self {
        Self {
            xtree: xot::Xot::new(),
            root: None,
            doc_el: None,
            apothrecipes_node: None,
            all_cures: Vec::new(),
        }
    }
}

impl ApothRecipeManager {

    pub fn get_full_cure_ids(&self) -> Vec<i32> {
        self.all_cures.iter().filter_map(|x| [CureResultType::BrandedCure,CureResultType::OffBrandCure].contains(&x.result_type).then_some(x.item_id)).collect()
    }

    pub fn get_partial_cure_ids(&self) -> Vec<i32> {
        self.all_cures.iter().filter_map(|x| [CureResultType::PartialCure].contains(&x.result_type).then_some(x.item_id)).collect()
    }

    pub fn get_failure_or_placebo_cure_ids(&self) -> Vec<i32> {
        self.all_cures.iter().filter_map(|x| [CureResultType::FailureCure, CureResultType::Placebo].contains(&x.result_type).then_some(x.item_id)).collect()
    }

    pub fn get_sideeffect_cure_ids(&self) -> Vec<i32> {
        self.all_cures.iter().filter_map(|x| (x.item_id_with_side_effects >= 0).then_some(x.item_id_with_side_effects)).collect()
    }

    pub fn is_not_full_cure_id(&self, uid: i32) -> bool {
        let mut known_duds: Vec<i32> = Vec::new();

        known_duds.extend(self.get_partial_cure_ids().iter());
        known_duds.extend(self.get_failure_or_placebo_cure_ids().iter());
        known_duds.extend(self.get_sideeffect_cure_ids().iter());

        known_duds.contains(&uid)
    }

    pub fn clear_data(&mut self) {
        self.root = None;
        self.doc_el = None;
        self.apothrecipes_node = None;
        self.all_cures.clear();
        self.xtree = xot::Xot::new();
    }

    pub fn load_data(&mut self, appconfig: &AppConfig) {

        let filepath_apothrecipes = PathBuf::from_iter([&appconfig.path_kynseed_data, &appconfig.filename_kynseed_apothrecipes]);

        let mut xml_vec = std::fs::read(&filepath_apothrecipes).unwrap();

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

        self.apothrecipes_node = Some(self.get_child_node_from_name(self.doc_el.unwrap(), "apothRecipes").unwrap());

        for child in self.xtree.children(self.apothrecipes_node.unwrap()) {
            match self.get_name_from_node( child) {
                None => continue,
                Some(child_el_name) => if child_el_name != "ApothRecipeSetup" {continue}
            };
            
            let child_result_type_node = self.get_child_node_from_name( child, "resultType").unwrap();
            let child_ailment_id_node = self.get_child_node_from_name( child, "AilmentID").unwrap();
            let child_item_id_node = self.get_child_node_from_name( child, "ItemID").unwrap();
            let child_item_id_with_side_effects_node = self.get_child_node_from_name( child, "ItemIDWithSideEffects").unwrap();

            let result_type = CureResultType::from_str(self.xtree.text_content_str(child_result_type_node).unwrap()).unwrap();
            let ailment_id = self.xtree.text_content_str(child_ailment_id_node).unwrap().parse::<i32>().unwrap();
            let item_id = self.xtree.text_content_str(child_item_id_node).unwrap().parse::<i32>().unwrap();
            let item_id_with_side_effects = self.xtree.text_content_str(child_item_id_with_side_effects_node).unwrap().parse::<i32>().unwrap();
            
            self.all_cures.push(ApothRecipe{result_type, ailment_id, item_id, item_id_with_side_effects});

        };
        
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