# Raft

Raft uses a stronger form of leader- ship than other consensus algorithms. For example, log entries only flow from the leader to other servers. This simplifies the management of the replicated log and makes Raft easier to understand

leaders are elected via: Raft uses randomized timers to elect leaders. This adds only a small amount of mechanism to the heartbeats already required for any consensus algorithm, while resolving conflicts simply and rapidly.

## "replicated state machines bad"

- if leader dies "op order" is also dead
  Distrobution of logs is the command structure

## Something

- Leader election
- Log replication
