POST http://127.0.0.1:7001/vote
Content-Type: application/json
{
  "RaftRequestVote": {
    "term": 0,
    "candidate_id": 1,
    "last_log_index": 0,
    "last_log_term": 0
  }
}
