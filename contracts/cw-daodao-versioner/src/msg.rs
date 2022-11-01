use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    QueryResult {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

/// We can import dao but for simplicity we show what we support
pub mod dao_registry {
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    pub mod Query {
        use super::State::*;
        use super::*;

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
        #[serde(rename_all = "snake_case", deny_unknown_fields)]
        pub enum QueryMsg {
            /// If version provided, tries to find given version. Otherwise returns
            /// the latest version registered.
            GetRegistration {
                registrar_addr:String,
                name: String,
                chain_id: String,
                version: Option<String>,
            },
            GetCodeIdInfo {
                registrar_addr: String,
                chain_id: String,
                code_id: u64,
            },
            ListRegistrations {
                registrar_addr: String,
                chain_id: String,
            },
        }
        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
        #[serde(rename_all = "snake_case", deny_unknown_fields)]
        pub struct GetRegistrationResponse {
            pub registration: Registration,
        }

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
        #[serde(rename_all = "snake_case", deny_unknown_fields)]
        pub struct ListRegistrationsResponse {
            pub registrations: Vec<Registration>,
        }
    }

    pub mod State {
        use super::*;

        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
        #[serde(rename_all = "snake_case", deny_unknown_fields)]
        pub struct Registration {
            pub contract_name: String,
            pub version: String,
            pub code_id: u64,
            pub checksum: String,
        }
    }
}
