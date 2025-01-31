use crate::{
    utils::{Author, BaseLog},
    Config,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};

pub trait ResearchLog: Serialize + for<'de> Deserialize<'de> {
    type Status: ToString;

    fn base(&self) -> &BaseLog;
    fn base_mut(&mut self) -> &mut BaseLog;
    fn status(&self) -> &Self::Status;
    fn status_mut(&mut self) -> &mut Self::Status;
    fn new(title: String, tags: HashSet<String>, author: Author) -> Self;
    fn update_status(&mut self, new_status: Self::Status);
    fn get_target_path(&self, config: &Config, current_path: &PathBuf) -> Result<PathBuf>;
}
