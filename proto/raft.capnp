@0x8f74f5b570af1aa4;

interface Raft {
#  struct AppendRequest {
#    id @0 :UInt64;
#    term @1 :UInt64;
#    leaderId @2 :UInt64;
#    prevLogIndex @3 :UInt64;
#    prevLogTerm @4 :UInt64;
#    entries @5 :List(Text); # change to List(Data)?
#  }
#	struct AppendResult {
#	  id @0 :UInt64;
#	  term @1 :UInt64;
#	  voteGranted @2 :Bool;
#	}
#
#	struct VoteRequest {
#	  id @0 :UInt64;
#	  term @1 :UInt64;
#	  candidateId @2 :UInt64;
#	  lastLogIndex @3 :UInt64;
#	  lastLogTerm @4 :UInt64;
#	}
#	struct VoteResult {
#	  id @0 :UInt64;
#	  term @1 :UInt64;
#	  success @2 :Bool;
#	}
#  appendEntries @0 (req :AppendRequest) -> (res :AppendResult);
#  requestVote @1 (req :VoteRequest) -> (res :VoteResult);

    struct HelloRequest {
        name @0 :Text;
    }

    struct HelloReply {
        message @0 :Text;
    }

    sayHello @0 (request: HelloRequest) -> (reply: HelloReply);
}
