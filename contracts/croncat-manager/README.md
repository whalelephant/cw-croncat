# CronCats task execution manager contract

Manager contract queries:

| Query           | Description                                    |
| --------------- | ---------------------------------------------- |
| Config          | Gets the manager contract configuration        |
| TreasuryBalance | Gets manager available balances                |
| UsersBalances   | Gets Cw20 balances of the given wallet address |
| TaskBalance     | Get task balance                               |


***

Manager contract actions:

| Execute           | Description                                                                           |
| ----------------- | ------------------------------------------------------------------------------------- |
| UpdateConfig      | Updates the manager config                                                            |
| ProxyCall         | Execute current task in the queue or task with queries if task_hash given             |
| RefillTaskBalance | Receive native coins to include them to the task                                      |
| Receive           | Receive cw20 coin                                                                     |
| CreateTaskBalance | Create task's balance, called by the tasks contract                                   |
| RemoveTask        | Remove task's balance, called by the tasks contract                                   |
| OwnerWithdraw     | Move balances from the manager to the owner address, or treasury_addr if set          |
| UserWithdraw      | Move balances from the manager to the owner address, or treasury_addr if set          |
| AgentWithdraw     | Withdraw agent rewards on agent removal, this should be called only by agent contract |
