static DEPLOYABLE: [&str; 2] = ["Ready for Deploy", "Completed"];

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Story {
    pub id: u64,
    pub name: String,
    pub workflow_state_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Workflow {
    pub id: u64,
    pub name: String,
    pub states: Vec<WorkflowState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowState {
    pub id: u64,
    pub name: String,
}

impl WorkflowState {
    /// From the name of the workflow state, determine whether this change is deployable
    pub fn is_deployable(&self) -> bool {
        DEPLOYABLE.iter().any(|s| s == &self.name)
    }
}
