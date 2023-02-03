# Smart contract for managing CronCats agents

Agents contract queries:

| Query      | Description |
| ----------- | ----------- |
| GetAgent      | Get an agent by specified account_id, returns AgentInfo if found       |
| GetAgentIds   | Gets the id list of agents, pagination is supported        |
| GetAgentTasks   | Gets the id list of agents, pagination is supported        |
| Config   | Gets the agent contract configuration        |
***

Agents contract actions:

| Query      | Description |
| ----------- | ----------- |
| RegisterAgent      | Action registers new agent      |
| UpdateAgent   | Action for updating agents        |
| CheckInAgent   | Action moves agent from pending to active list        |
| UnregisterAgent   | Actions for removing agent from the system        |
| UpdateConfig   | Action for updating agent contract configuration        |