use anyhow::{bail, Result};
use convert_case::{Case, Casing};
use log::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::ir::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpandExtends {}

impl ExpandExtends {
    pub fn run(&self, ir: &mut IR) -> anyhow::Result<()> {
        let mut rename_fs = HashMap::new();
        let mut rename_enum = HashMap::new();

        // Expand blocks
        let deps = ir
            .blocks
            .iter()
            .map(|(k, v)| (k.clone(), v.extends.clone()))
            .collect();
        let mut pending_removal = HashSet::new();
        for name in topological_sort(deps)? {
            let extends = ir.blocks.get(&name).unwrap().extends.clone();
            if let Some(parent_name) = &extends {
                pending_removal.insert(parent_name.clone());
                let parent = ir.blocks.get(&parent_name.clone()).unwrap();

                let items = parent
                    .items
                    .iter()
                    .map(|x| {
                        let mut item = x.clone();
                        if let BlockItem {
                            inner: BlockItemInner::Register(reg),
                            ..
                        } = &mut item
                        {
                            if let Some(ref mut fieldset) = &mut reg.fieldset {
                                let parent_mod = parent_name.to_case(Case::Snake);
                                let cur_mod = name.split("::").next().unwrap();
                                let new = fieldset.replace(&parent_mod, cur_mod);
                                rename_fs.insert(fieldset.clone(), new.clone());
                                if let Some(fieldset) = ir.fieldsets.get_mut(fieldset) {
                                    for field in fieldset.fields.iter_mut() {
                                        if let Some(enumm) = &field.enumm {
                                            let new = enumm.replace(&parent_mod, cur_mod);
                                            rename_enum.insert(enumm.clone(), new.clone());
                                            field.enumm = Some(new);
                                        }
                                    }
                                }
                                *fieldset = new;
                            }
                        }

                        item
                    })
                    .collect::<Vec<_>>();
                let parent_description = parent.description.clone();
                let block = ir.blocks.get_mut(&name.clone()).unwrap();

                if block.description.is_none() {
                    block.description = parent_description;
                }

                for i in items {
                    if !block.items.iter().any(|j| j.name == i.name) {
                        block.items.push(i);
                    }
                }
            }
        }
        for dep in pending_removal {
            ir.blocks.remove(dep.as_str());
        }

        // Expand fieldsets
        let deps = ir
            .fieldsets
            .iter()
            .map(|(k, v)| (k.clone(), v.extends.clone()))
            .collect();
        let mut pending_removal = HashSet::new();
        for name in topological_sort(deps)? {
            let fieldset = ir.fieldsets.get(&name).unwrap();
            if let Some(parent_name) = &fieldset.extends {
                pending_removal.insert(parent_name.clone());
                let parent_fieldset = ir.fieldsets.get(parent_name).unwrap();

                let items = parent_fieldset
                    .fields
                    .iter()
                    .map(|x| {
                        let parent_mod = parent_name.split("::").next().unwrap();
                        let cur_mod = name.split("::").next().unwrap();

                        let mut item = x.clone();

                        if let Some(ref mut enumm) = &mut item.enumm {
                            if enumm.starts_with(parent_mod) {
                                let new = enumm.replace(parent_mod, cur_mod);
                                rename_enum.insert(enumm.clone(), new.clone());
                                *enumm = new;
                            }
                        }
                        item
                    })
                    .collect::<Vec<_>>();

                let fieldset = ir.fieldsets.get_mut(&name).unwrap();

                for i in items {
                    if !fieldset.fields.iter().any(|j| j.name == i.name) {
                        fieldset.fields.push(i);
                    }
                }
            }
        }
        for dep in pending_removal {
            ir.fieldsets.remove(dep.as_str());
        }

        for (old, new) in rename_fs.into_iter() {
            if let Some(val) = ir.fieldsets.remove(&old) {
                ir.fieldsets.insert(new, val);
            }
        }

        for (old, new) in rename_enum.into_iter() {
            if let Some(val) = ir.enums.remove(&old) {
                ir.enums.insert(new, val);
            }
        }

        Ok(())
    }
}

fn topological_sort(vals: BTreeMap<String, Option<String>>) -> Result<Vec<String>> {
    for (name, dep) in &vals {
        info!("{:?} → {:?}", name, dep);
    }

    let mut done = BTreeSet::new();
    let mut res = Vec::new();

    while done.len() != vals.len() {
        for (name, dep) in &vals {
            if done.contains(name) {
                continue;
            }
            if let Some(dep) = dep {
                if !vals.contains_key(dep) {
                    bail!("Couldn't resolve dependency for {name} → {dep}");
                }
                if !done.contains(dep) {
                    continue;
                }
            }
            info!("doing {:?} ", name);
            done.insert(name.clone());
            res.push(name.clone());
        }
    }

    Ok(res)
}
