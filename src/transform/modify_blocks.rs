use serde::{Deserialize, Serialize};

use super::common::*;
use crate::ir::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModifyBlocks {
    pub blocks: RegexSet,
    pub description: Option<String>,
}

impl ModifyBlocks {
    pub fn run(&self, ir: &mut IR) -> anyhow::Result<()> {
        for id in match_all(ir.blocks.keys().cloned(), &self.blocks) {
            let block = get_mut!(ir, blocks, &id)?;

            if let Some(description) = &self.description {
                block.description = Some(description.clone());
            }
        }

        Ok(())
    }
}
