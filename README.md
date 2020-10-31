# Rust KVS

- Should put a bit more thought into what will go in the README, maybe add some
  bades?

## Terminology

---

- Inspired by [bitcask](https://github.com/basho/bitcask) apparently.

- Lesson learned: after struggling with passing the tests for `project-2`,
  thought I'd share what the issue is. Although the kv store worked in practice,
  in the tests I kept having flakey behaviour -- individually, the tests would
  sometime pass, if run in batch they'd fail. The tests that failed were related
  to overwriting values, getting stored values, removing stored, etc. In
  essence, it all had to do with how we replay the log. The _issue_ turned to be
  worse than I though; it was related to the directory logic. Because we opened
  the directory and dumped all files I thought it would all work well, however,
  the temp directory used to run the tests didn't agree. The first part of the
  tests would always pass (it assumed no prev log file to open so we just
  created one) but the subsequent steps would always fail, this is because the
  file was never found a new one created! Phew, got solved.
