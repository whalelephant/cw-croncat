use cosmwasm_std::Binary;
use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cw_rules_core::{
    msg::{ExecuteMsg, InstantiateMsg, QueryConstruct, QueryMsg, QueryMultiResponse, RuleResponse},
    types::{CheckOwnerOfNft, CheckProposalStatus, HasBalanceGte, Rule, Status},
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("packages");
    out_dir.push("cw-rules-core");
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMultiResponse), &out_dir);
    export_schema(&schema_for!(QueryConstruct), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Rule), &out_dir);
    export_schema(&schema_for!(HasBalanceGte), &out_dir);
    export_schema(&schema_for!(CheckOwnerOfNft), &out_dir);
    export_schema(&schema_for!(CheckProposalStatus), &out_dir);

    export_schema_with_title(
        &schema_for!(RuleResponse<Option<Binary>>),
        &out_dir,
        "RuleResponse",
    );
    export_schema_with_title(
        &schema_for!((bool, Option<u64>)),
        &out_dir,
        "QueryConstructResponse",
    );

    export_schema_with_title(
        &schema_for!((bool, Option<Binary>)),
        &out_dir,
        "GetBalanceResponse",
    );
    export_schema_with_title(
        &schema_for!((bool, Option<Binary>)),
        &out_dir,
        "GetCw20BalanceResponse",
    );
    export_schema_with_title(
        &schema_for!((bool, Option<Binary>)),
        &out_dir,
        "CheckOwnerOfNftResponse",
    );
    export_schema_with_title(
        &schema_for!((bool, Option<Binary>)),
        &out_dir,
        "HasBalanceGteResponse",
    );
    export_schema_with_title(
        &schema_for!((bool, Option<Binary>)),
        &out_dir,
        "CheckProposalStatusResponse",
    );
    export_schema_with_title(
        &schema_for!((bool, Option<Binary>)),
        &out_dir,
        "GenericQueryResponse",
    );
    export_schema_with_title(
        &schema_for!((bool, Option<Binary>)),
        &out_dir,
        "HasBalanceGteResponse",
    );
    export_schema_with_title(
        &schema_for!((bool, Option<Binary>)),
        &out_dir,
        "CheckProposalStatusResponse",
    );
    export_schema_with_title(
        &schema_for!((bool, Option<Binary>)),
        &out_dir,
        "CheckProposalStatusResponse",
    );
}
