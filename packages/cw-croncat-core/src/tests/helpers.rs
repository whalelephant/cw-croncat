pub fn get_contract_version() -> String {
    const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
    CONTRACT_VERSION.to_owned()
}
