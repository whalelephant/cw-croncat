# croncat-factory
Factory contract for deploying, migrating and storing croncat contracts metadata. It manages versions of all contracts, helping with backward compatibility and DAO governed contracts.

For more info see the [wiki](https://github.com/CronCats/cw-croncat/wiki/%F0%9F%8F%AD-Factory-Architecture)


Factory contract queries:

| Query                  | Description                                               |
| ---------------------- | --------------------------------------------------------- |
| Config                 | Gets the factory contract configuration                   |
| LatestContracts        | Gets latest contract names and metadatas of the contracts |
| LatestContract         | Gets latest version metadata of the contract              |
| VersionsByContractName | Gets metadatas of the contract                            |
| ContractNames          | Gets list of the contract names                           |
| AllEntries             | Gets all contract names and metadatas stored in factory   |

***

Factory contract actions:

| Execute         | Description                                                                                 |
| --------------- | ------------------------------------------------------------------------------------------- |
| UpdateConfig    | Updates the factory config                                                                  |
| Remove          | Removes contract metadata from the factory if contract is paused or it is library contract. |
| UpdateMetadata  | Update fields of the contract metadata                                                      |
| UnregisterAgent | Actions for removing agent from the system                                                  |
