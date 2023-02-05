# CronCats tasks contract

Tasks contract queries:

| Query                  | Description                                |
| ---------------------- | ------------------------------------------ |
| Config                 | Gets the tasks contract configuration      |
| CurrentTaskInfo        | Gets current task information              |
| TasksWithQueriesTotal  | Get the total amount of tasks with queries |
| Tasks                  | Get list of active tasks, without queries  |
| TasksWithQueries       | Get list of active tasks, with queries     |
| Task                   | Simulate task_hash by the given task       |
| SlotHashes             | Get slot hashes by given slot              |
| SlotIds                | Get active slots                           |
| CurrentTask            | Get next task to be done                   |
| CurrentTaskWithQueries | Get task with queries if it's ready        |
| TasksByOwner           | Get tasks created by the given address     |


***

Tasks contract actions:

| Execute             | Description                                                                              |
| ------------------- | ---------------------------------------------------------------------------------------- |
| UpdateConfig        | Updates the tasks contract config                                                        |
| CreateTask          | Allows any user or contract to pay for future txns based on a specific schedule contract |
| RemoveTask          | Deletes a task in its entirety, returning any remaining balance to task owner            |
| RemoveTaskByManager | Remove task, used by the manager if task reached it's stop condition                     |
| RescheduleTask      | Try to reschedule a task, if possible, used by the manager                               |
