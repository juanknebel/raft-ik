POST http://127.0.0.1:7001/heartbeat
Content-Type: application/json
{
  "Heartbeat": {
    "message": {
      "term": 0,
      "leader_id": 1,
      "prev_log_index": 0,
      "prev_log_term": 0,
      "leader_commit": 0
    }
  }
}
