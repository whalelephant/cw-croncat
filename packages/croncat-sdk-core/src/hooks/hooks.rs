
use thiserror::Error;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    Addr, CustomQuery, Deps, StdError, StdResult, Storage, SubMsg,
};
use cw_storage_plus::{Map};
#[cw_serde]
pub struct HooksResponse {
    pub hooks: Vec<String>,
}

#[derive(Error, Debug, PartialEq)]
pub enum HookError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Given address already registered as a hook")]
    HookAlreadyRegistered {},

    #[error("Given address not registered as a hook")]
    HookNotRegistered {},
}

// store all hook addresses in one item. We cannot have many of them before the contract becomes unusable anyway.
pub struct Hooks<'a>(Map<'a, &'a str, Vec<Addr>>);

impl<'a> Hooks<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        Hooks(Map::new(storage_key))
    }

    pub fn add_hook(
        &self,
        storage: &mut dyn Storage,
        prefix: &str,
        addr: Addr,
    ) -> Result<(), HookError> {
        let mut hooks = self.0.may_load(storage, prefix.clone())?.unwrap_or_default();
        if !hooks.iter().any(|h| h == &addr) {
            hooks.push(addr);
        } else {
            return Err(HookError::HookAlreadyRegistered {});
        }
        Ok(self.0.save(storage, prefix, &hooks)?)
    }

    pub fn remove_hooks(&self, storage: &mut dyn Storage, prefix: &str) -> Result<(), HookError> {
        self.0
            .may_load(storage, prefix.clone())?
            .ok_or(HookError::HookNotRegistered {})?;
        self.0.remove(storage, prefix); //remove all hooks by prefix
        Ok(())
    }
    pub fn remove_hook(
        &self,
        storage: &mut dyn Storage,
        prefix: &str,
        addr: Addr,
    ) -> Result<(), HookError> {
        let mut hooks = self
            .0
            .may_load(storage, prefix.clone())?
            .ok_or(HookError::HookNotRegistered {})
            .unwrap();
        if let Some(p) = hooks.iter().position(|x| x == &addr) {
            hooks.remove(p);
        } else {
            return Err(HookError::HookNotRegistered {});
        }
        Ok(self.0.save(storage, prefix, &hooks)?)
    }
    pub fn prepare_hooks<F: Fn(Addr) -> StdResult<SubMsg>>(
        &self,
        storage: &dyn Storage,
        prefix: &str,
        prep: F,
    ) -> StdResult<Vec<SubMsg>> {
        self.0
            .may_load(storage, prefix)?
            .unwrap_or_default()
            .into_iter()
            .map(prep)
            .collect()
    }

    pub fn query_hooks<Q: CustomQuery>(
        &self,
        deps: Deps<Q>,
        prefix: &str,
    ) -> StdResult<HooksResponse> {
        let hooks = self.0.may_load(deps.storage, prefix)?.unwrap_or_default();
        let hooks = hooks.into_iter().map(String::from).collect();
        Ok(HooksResponse { hooks })
    }

    // Return true if hook is in hooks
    pub fn query_hook<Q: CustomQuery>(
        &self,
        deps: Deps<Q>,
        prefix: &str,
        hook: String,
    ) -> StdResult<bool> {
        Ok(self
            .query_hooks(deps, prefix)?
            .hooks
            .into_iter()
            .any(|h| h == hook))
    }
}

// TODO: add test coverage
