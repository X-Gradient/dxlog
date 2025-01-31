// crates/dxlog/src/reference.rs
use crate::{
    load_config, research_log::ResearchLog, HypothesisManager, HypothesisStatus, KnowledgeManager,
    KnowledgeStatus, LiteratureManager, LiteratureStatus,
};
use anyhow::Result;
use std::collections::HashSet;
use uuid::Uuid;

pub struct ReferenceInfo {
    pub id: String,
    pub type_: String,
    pub title: String,
    pub tags: HashSet<String>,
}

pub fn add_reference(source_id: &str, target_id: &str) -> Result<()> {
    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());

    let target_uuid = Uuid::parse_str(target_id)?;

    if let Ok((mut log, path)) = h_manager.find(source_id) {
        log.base_mut().references.insert(target_uuid);
        h_manager.manager.update_log(&mut log, &path)
    } else if let Ok((mut log, path)) = l_manager.find(source_id) {
        log.base_mut().references.insert(target_uuid);
        l_manager.manager.update_log(&mut log, &path)
    } else if let Ok((mut log, path)) = k_manager.find(source_id) {
        log.base_mut().references.insert(target_uuid);
        k_manager.manager.update_log(&mut log, &path)
    } else {
        Err(anyhow::anyhow!("Source log not found"))
    }
}

fn is_reference_complete(target_id: &str) -> Result<bool> {
    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());

    if let Ok((log, _)) = h_manager.find(target_id) {
        Ok(matches!(
            log.status,
            HypothesisStatus::Proven | HypothesisStatus::Disproven | HypothesisStatus::Inconclusive
        ))
    } else if let Ok((log, _)) = l_manager.find(target_id) {
        Ok(matches!(log.status, LiteratureStatus::Completed))
    } else if let Ok((log, _)) = k_manager.find(target_id) {
        Ok(matches!(log.status, KnowledgeStatus::Published))
    } else {
        Err(anyhow::anyhow!("Reference not found"))
    }
}

pub fn force_add_reference(source_id: &str, target_id: &str) -> Result<()> {
    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());

    let target_uuid = Uuid::parse_str(target_id)?;

    if !is_reference_complete(target_id)? {
        return Err(anyhow::anyhow!(
            "Warning: Referenced research log is not in a complete state (proven, completed, or published). References should ideally point to completed research."
        ));
    }

    if let Ok((mut log, path)) = h_manager.find(source_id) {
        log.base_mut().references.insert(target_uuid);
        h_manager.manager.update_log(&mut log, &path)
    } else if let Ok((mut log, path)) = l_manager.find(source_id) {
        log.base_mut().references.insert(target_uuid);
        l_manager.manager.update_log(&mut log, &path)
    } else if let Ok((mut log, path)) = k_manager.find(source_id) {
        log.base_mut().references.insert(target_uuid);
        k_manager.manager.update_log(&mut log, &path)
    } else {
        Err(anyhow::anyhow!("Source log not found"))
    }
}

pub fn remove_reference(source_id: &str, target_id: &str) -> Result<()> {
    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());

    let target_uuid = Uuid::parse_str(target_id)?;

    if let Ok((mut log, path)) = h_manager.find(source_id) {
        log.base_mut().references.remove(&target_uuid);
        h_manager.manager.update_log(&mut log, &path)
    } else if let Ok((mut log, path)) = l_manager.find(source_id) {
        log.base_mut().references.remove(&target_uuid);
        l_manager.manager.update_log(&mut log, &path)
    } else if let Ok((mut log, path)) = k_manager.find(source_id) {
        log.base_mut().references.remove(&target_uuid);
        k_manager.manager.update_log(&mut log, &path)
    } else {
        Err(anyhow::anyhow!("Source log not found"))
    }
}

pub fn list_references(id: &str) -> Result<Vec<ReferenceInfo>> {
    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());

    let referenced_ids = if let Ok((log, _)) = h_manager.find(id) {
        log.base().references.clone()
    } else if let Ok((log, _)) = l_manager.find(id) {
        log.base().references.clone()
    } else if let Ok((log, _)) = k_manager.find(id) {
        log.base().references.clone()
    } else {
        return Err(anyhow::anyhow!("Log not found"));
    };

    let mut references = Vec::new();
    for ref_id in referenced_ids {
        let short_id = ref_id.to_string();
        if let Ok((log, _)) = h_manager.find(&short_id) {
            references.push(ReferenceInfo {
                id: short_id,
                type_: "hypothesis".to_string(),
                title: log.base().title.clone(),
                tags: log.base().tags.clone(),
            });
        } else if let Ok((log, _)) = l_manager.find(&short_id) {
            references.push(ReferenceInfo {
                id: short_id,
                type_: "literature".to_string(),
                title: log.base().title.clone(),
                tags: log.base().tags.clone(),
            });
        } else if let Ok((log, _)) = k_manager.find(&short_id) {
            references.push(ReferenceInfo {
                id: short_id,
                type_: "knowledge".to_string(),
                title: log.base().title.clone(),
                tags: log.base().tags.clone(),
            });
        }
    }

    Ok(references)
}
