use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cw_croncat_core::msg::{
    AgentResponse, Croncat, CwCroncatResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
    SimulateTaskResponse, TaskResponse, TaskWithQueriesResponse,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("packages");
    out_dir.push("cw-croncat-core");
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Croncat), &out_dir);
    export_schema_with_title(&schema_for!(bool), &out_dir, "ValidateIntervalResponse");
    export_schema_with_title(
        &schema_for!(Option<AgentResponse>),
        &out_dir,
        "GetAgentResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<TaskResponse>),
        &out_dir,
        "GetTasksResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<TaskResponse>),
        &out_dir,
        "GetTasksByOwnerResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<TaskWithQueriesResponse>),
        &out_dir,
        "GetTasksWithQueriesResponse",
    );
    export_schema_with_title(
        &schema_for!(Option<TaskResponse>),
        &out_dir,
        "GetTaskResponse",
    );
    export_schema_with_title(&schema_for!(String), &out_dir, "GetTaskHashResponse");
    export_schema_with_title(
        &schema_for!(Option<TaskResponse>),
        &out_dir,
        "GetAgentTasksResponse",
    );
    export_schema_with_title(
        &schema_for!(CwCroncatResponse),
        &out_dir,
        "GetStateResponse",
    );
    export_schema_with_title(
        &schema_for!(SimulateTaskResponse),
        &out_dir,
        "GasSimulationResponse",
    );
    export_schema_with_title(
        &schema_for!(SimulateTaskResponse),
        &out_dir,
        "SimulateTaskResponse",
    );
    export_schema_with_title(
        &schema_for!(Vec<SimulateTaskResponse>),
        &out_dir,
        "SimulateTaskResponse",
    );
}
