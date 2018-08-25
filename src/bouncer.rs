use clubhouse;

#[derive(Serialize, Deserialize, Debug)]
pub struct Deployable {
    pub deployable: bool,
    pub story_states: Vec<StoryState>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryDeployable {
    pub story_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoryState {
    pub deployable: bool,
    pub story: clubhouse::Story,
    pub state: clubhouse::WorkflowState,
}
