use super::BaseLog;
use anyhow::Result;
use std::collections::HashSet;
use uuid::Uuid;

pub fn detect_cycles(_references: &HashSet<Uuid>, new_ref: Uuid, logs: &[BaseLog]) -> bool {
    let mut visited = HashSet::new();
    let mut stack = vec![new_ref];

    while let Some(current) = stack.pop() {
        if !visited.insert(current) {
            return true;
        }
        if let Some(log) = logs.iter().find(|l| l.id == current) {
            stack.extend(log.references.iter());
        }
    }
    false
}

pub fn add_reference(log: &mut BaseLog, ref_id: Uuid, all_logs: &[BaseLog]) -> Result<()> {
    if detect_cycles(&log.references, ref_id, all_logs) {
        return Err(anyhow::anyhow!(
            "Adding this reference would create a cycle"
        ));
    }
    log.references.insert(ref_id);
    Ok(())
}

pub fn remove_reference(log: &mut BaseLog, ref_id: &Uuid) {
    log.references.remove(ref_id);
}