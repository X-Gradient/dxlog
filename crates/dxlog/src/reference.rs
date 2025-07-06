// crates/dxlog/src/reference.rs
use crate::{
    load_config, research_log::ResearchLog, utils::{self, BaseLog}, HypothesisManager, HypothesisStatus, KnowledgeManager,
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

fn collect_all_logs() -> Result<Vec<BaseLog>> {
    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());
    
    let mut all_logs = Vec::new();
    
    // Use the existing get_all_base_logs method to avoid duplication
    all_logs.extend(h_manager.manager.get_all_base_logs()?);
    all_logs.extend(l_manager.manager.get_all_base_logs()?);
    all_logs.extend(k_manager.manager.get_all_base_logs()?);
    
    Ok(all_logs)
}

fn validate_target_exists(target_id: &str) -> Result<()> {
    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());
    
    // Try to find the target in any of the managers
    if h_manager.manager.find(target_id).is_ok() 
        || l_manager.manager.find(target_id).is_ok() 
        || k_manager.manager.find(target_id).is_ok() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Target reference '{}' not found", target_id))
    }
}

pub fn add_reference(source_id: &str, target_id: &str) -> Result<()> {
    let target_uuid = Uuid::parse_str(target_id)?;
    
    // Validate target exists
    validate_target_exists(target_id)?;
    
    // Check if target is in complete state (warn but don't fail)
    if !is_reference_complete(target_id)? {
        return Err(anyhow::anyhow!(
            "Warning: Referenced research log is not in a complete state (proven, completed, or published). References should ideally point to completed research."
        ));
    }

    // Collect all logs for cycle detection
    let all_logs = collect_all_logs()?;

    // Check for cycles before adding
    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());

    if let Ok((log, _)) = h_manager.manager.find(source_id) {
        if utils::detect_cycles(&log.base().references, target_uuid, &all_logs) {
            return Err(anyhow::anyhow!("Adding this reference would create a cycle"));
        }
        h_manager.manager.add_reference(source_id, target_uuid)
    } else if let Ok((log, _)) = l_manager.manager.find(source_id) {
        if utils::detect_cycles(&log.base().references, target_uuid, &all_logs) {
            return Err(anyhow::anyhow!("Adding this reference would create a cycle"));
        }
        l_manager.manager.add_reference(source_id, target_uuid)
    } else if let Ok((log, _)) = k_manager.manager.find(source_id) {
        if utils::detect_cycles(&log.base().references, target_uuid, &all_logs) {
            return Err(anyhow::anyhow!("Adding this reference would create a cycle"));
        }
        k_manager.manager.add_reference(source_id, target_uuid)
    } else {
        Err(anyhow::anyhow!("Source log not found"))
    }
}

fn is_reference_complete(target_id: &str) -> Result<bool> {
    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());

    if let Ok((log, _)) = h_manager.manager.find(target_id) {
        Ok(matches!(
            log.status,
            HypothesisStatus::Proven | HypothesisStatus::Disproven | HypothesisStatus::Inconclusive
        ))
    } else if let Ok((log, _)) = l_manager.manager.find(target_id) {
        Ok(matches!(log.status, LiteratureStatus::Completed))
    } else if let Ok((log, _)) = k_manager.manager.find(target_id) {
        Ok(matches!(log.status, KnowledgeStatus::Published))
    } else {
        Err(anyhow::anyhow!("Reference not found"))
    }
}

pub fn force_add_reference(source_id: &str, target_id: &str) -> Result<()> {
    let target_uuid = Uuid::parse_str(target_id)?;
    
    // Validate target exists
    validate_target_exists(target_id)?;

    // Collect all logs for cycle detection (still enforce cycle prevention even in force mode)
    let all_logs = collect_all_logs()?;

    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());

    if let Ok((log, _)) = h_manager.manager.find(source_id) {
        // Check for cycles before adding (even in force mode)
        if utils::detect_cycles(&log.base().references, target_uuid, &all_logs) {
            return Err(anyhow::anyhow!("Adding this reference would create a cycle"));
        }
        h_manager.manager.add_reference(source_id, target_uuid)
    } else if let Ok((log, _)) = l_manager.manager.find(source_id) {
        // Check for cycles before adding (even in force mode)
        if utils::detect_cycles(&log.base().references, target_uuid, &all_logs) {
            return Err(anyhow::anyhow!("Adding this reference would create a cycle"));
        }
        l_manager.manager.add_reference(source_id, target_uuid)
    } else if let Ok((log, _)) = k_manager.manager.find(source_id) {
        // Check for cycles before adding (even in force mode)
        if utils::detect_cycles(&log.base().references, target_uuid, &all_logs) {
            return Err(anyhow::anyhow!("Adding this reference would create a cycle"));
        }
        k_manager.manager.add_reference(source_id, target_uuid)
    } else {
        Err(anyhow::anyhow!("Source log not found"))
    }
}

pub fn remove_reference(source_id: &str, target_id: &str) -> Result<()> {
    let target_uuid = Uuid::parse_str(target_id)?;

    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());

    if h_manager.manager.find(source_id).is_ok() {
        h_manager.manager.remove_reference(source_id, target_uuid)
    } else if l_manager.manager.find(source_id).is_ok() {
        l_manager.manager.remove_reference(source_id, target_uuid)
    } else if k_manager.manager.find(source_id).is_ok() {
        k_manager.manager.remove_reference(source_id, target_uuid)
    } else {
        Err(anyhow::anyhow!("Source log not found"))
    }
}

pub fn list_references(id: &str) -> Result<Vec<ReferenceInfo>> {
    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());

    let referenced_ids = if let Ok((log, _)) = h_manager.manager.find(id) {
        log.base().references.clone()
    } else if let Ok((log, _)) = l_manager.manager.find(id) {
        log.base().references.clone()
    } else if let Ok((log, _)) = k_manager.manager.find(id) {
        log.base().references.clone()
    } else {
        return Err(anyhow::anyhow!("Log not found"));
    };

    let mut references = Vec::new();
    for ref_id in referenced_ids {
        let short_id = ref_id.to_string();
        if let Ok((log, _)) = h_manager.manager.find(&short_id) {
            references.push(ReferenceInfo {
                id: short_id,
                type_: "hypothesis".to_string(),
                title: log.base().title.clone(),
                tags: log.base().tags.clone(),
            });
        } else if let Ok((log, _)) = l_manager.manager.find(&short_id) {
            references.push(ReferenceInfo {
                id: short_id,
                type_: "literature".to_string(),
                title: log.base().title.clone(),
                tags: log.base().tags.clone(),
            });
        } else if let Ok((log, _)) = k_manager.manager.find(&short_id) {
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

pub fn find_referencing_items(target_id: &str) -> Result<Vec<ReferenceInfo>> {
    let target_uuid = Uuid::parse_str(target_id)?;
    let config = load_config()?;
    let h_manager = HypothesisManager::new(config.clone());
    let l_manager = LiteratureManager::new(config.clone());
    let k_manager = KnowledgeManager::new(config.clone());
    
    let mut referencing_items = Vec::new();
    
    // Check all hypotheses
    let hypotheses = h_manager.manager.log_manager.list_logs(None, None)?;
    for hypothesis in hypotheses {
        if hypothesis.base().references.contains(&target_uuid) {
            referencing_items.push(ReferenceInfo {
                id: hypothesis.base().id.to_string(),
                type_: "hypothesis".to_string(),
                title: hypothesis.base().title.clone(),
                tags: hypothesis.base().tags.clone(),
            });
        }
    }
    
    // Check all literature
    let literature_items = l_manager.manager.log_manager.list_logs(None, None)?;
    for literature in literature_items {
        if literature.base().references.contains(&target_uuid) {
            referencing_items.push(ReferenceInfo {
                id: literature.base().id.to_string(),
                type_: "literature".to_string(),
                title: literature.base().title.clone(),
                tags: literature.base().tags.clone(),
            });
        }
    }
    
    // Check all knowledge
    let knowledge_items = k_manager.manager.log_manager.list_logs(None, None)?;
    for knowledge in knowledge_items {
        if knowledge.base().references.contains(&target_uuid) {
            referencing_items.push(ReferenceInfo {
                id: knowledge.base().id.to_string(),
                type_: "knowledge".to_string(),
                title: knowledge.base().title.clone(),
                tags: knowledge.base().tags.clone(),
            });
        }
    }
    
    Ok(referencing_items)
}
