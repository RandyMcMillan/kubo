# kubo-ffi exposure plan

## Current state, measured and itemized

Legend: ✅ exposed + wrapped in Rust · 🔶 Go export exists, no Rust
wrapper (or partial) · ⬜ not exposed at all.

### A. Go `//export` layer — 67 exports, 38 wrapped (57%)

Node lifecycle / config — all wrapped:
1. ✅ kubo_version
2. ✅ kubo_get_logs
3. ✅ kubo_init_repo
4. ✅ kubo_init_repo_networked
5. ✅ kubo_init_repo_with_config
6. ✅ kubo_node_start
7. ✅ kubo_node_stop
8. ✅ kubo_node_peer_id
9. ✅ kubo_node_id
10. ✅ kubo_node_listening_addrs
11. ✅ kubo_node_api_addrs
12. ✅ kubo_node_connect
13. ✅ kubo_swarm_peers
14. ✅ kubo_node_start_api
15. ✅ kubo_node_start_gateway
16. ✅ kubo_node_add_static
17. ✅ kubo_node_add_crate_redirect
18. ✅ kubo_node_set_auth_token
19. ✅ kubo_watch_path
20. ✅ kubo_watch_stop
21. ✅ kubo_unixfs_add_bytes
22. ✅ kubo_unixfs_cat

Unwrapped (29 = 43% of the Go layer):
23. 🔶 kubo_block_put
24. 🔶 kubo_block_get
25. 🔶 kubo_block_stat
26. 🔶 kubo_git_clone
27. 🔶 kubo_git_open
28. 🔶 kubo_git_init
29. 🔶 kubo_git_repo_head
30. 🔶 kubo_git_repo_free
31. 🔶 kubo_git_repo_is_bare
32. 🔶 kubo_git_repo_branches
33. 🔶 kubo_git_repo_remotes
34. 🔶 kubo_git_repo_create_branch
35. 🔶 kubo_git_repo_commit_lookup
36. 🔶 kubo_git_repo_tree_entries
37. 🔶 kubo_git_repo_blob_read
38. 🔶 kubo_git_repo_status
39. 🔶 kubo_git_repo_diff_trees
40. 🔶 kubo_git_push
41. 🔶 kubo_nostr_event_verify
42. 🔶 kubo_nostr_nip19_encode_pubkey
43. 🔶 kubo_nostr_nip19_decode_pubkey
44. 🔶 kubo_nostr_nip19_encode_seckey
45. 🔶 kubo_nostr_nip19_decode_seckey
46. 🔶 kubo_nostr_nip19_encode_note
47. 🔶 kubo_nostr_nip19_decode_note
48. 🔶 kubo_nostr_nip19_encode_entity
49. 🔶 kubo_nostr_nip19_decode_entity
50. 🔶 kubo_nostr_nip05_verify
51. 🔶 kubo_nostr_nip05_query

Wrapped nostr/git:
52. ✅ kubo_nostr_generate_key
53. ✅ kubo_nostr_get_public_key
54. ✅ kubo_nostr_event_sign
55. ✅ kubo_nostr_event_sign_with_tags
56. ✅ kubo_nostr_relay_connect
57. ✅ kubo_nostr_relay_close
58. ✅ kubo_nostr_relay_publish
59. ✅ kubo_nostr_relay_subscribe
60. ✅ kubo_nostr_relay_drain
61. ✅ kubo_nostr_relay_unsubscribe
62. ✅ kubo_nostr_nsec_to_swarm_key
63. ✅ kubo_swarm_key_generate
64. ✅ kubo_git_fetch_all
65. ✅ kubo_git_blame
66. ✅ kubo_git_log
67. ✅ kubo_git_tags

(Rust-only helpers with no Go export: run_command, run, run_on_node.)

### B. Kubo CoreAPI layer — 40 methods, 7 exposed in any form (17.5%)

Unixfs:
1. 🔶 Unixfs.Add — bytes only, no file/dir/progress
2. ✅ Unixfs.Get
3. ⬜ Unixfs.Ls

Block:
4. 🔶 Block.Put — Go only, unwrapped
5. 🔶 Block.Get — Go only, unwrapped
6. 🔶 Block.Stat — Go only, unwrapped
7. ⬜ Block.Rm

Dag:
8. ⬜ Dag (import/export/put/get — whole service)

Name (IPNS):
9. ⬜ Name.Publish
10. ⬜ Name.Resolve
11. ⬜ Name.Search

Key:
12. ⬜ Key.Generate
13. ⬜ Key.List
14. ⬜ Key.Self
15. ⬜ Key.Rename
16. ⬜ Key.Remove

Pin:
17. ⬜ Pin.Add
18. ⬜ Pin.Ls
19. ⬜ Pin.IsPinned
20. ⬜ Pin.Rm
21. ⬜ Pin.Update
22. ⬜ Pin.Verify

Object:
23. ⬜ Object.AddLink
24. ⬜ Object.RmLink
25. ⬜ Object.Diff

Swarm:
26. ✅ Swarm.Connect
27. ✅ Swarm.Peers
28. ⬜ Swarm.Disconnect
29. ⬜ Swarm.KnownAddrs
30. ⬜ Swarm.LocalAddrs
31. ⬜ Swarm.ListenAddrs

PubSub:
32. ⬜ PubSub.Publish
33. ⬜ PubSub.Subscribe
34. ⬜ PubSub.Ls
35. ⬜ PubSub.Peers

Routing:
36. ⬜ Routing.Get
37. ⬜ Routing.Put
38. ⬜ Routing.FindPeer
39. ⬜ Routing.FindProviders
40. ⬜ Routing.Provide

### Totals

| Measure | Done | Total | Percent |
|---|---|---|---|
| Go exports wrapped in Rust | 38 | 67 | 57% |
| CoreAPI methods exposed (any form) | 7 | 40 | 17.5% |
| **Entire FFI exposure** | **38** | **100** | **38%** |

## Phases

- **Phase 0 — Rust parity:** wrap items 23–51 (29 wrappers, mechanical).
- **Batch A:** CoreAPI 3, 8, 17–22 (Pin ×6, Dag, Unixfs.Ls) — unblocks WebUI.
- **Batch B:** CoreAPI 28–31 (Swarm completion).
- **Batch C:** CoreAPI 9–16 (IPNS + Key).
- **Batch D:** CoreAPI 23–25 (Object).
- **Batch E:** CoreAPI 32–40 (PubSub + Routing).
- **Batch F:** repo stat/gc/verify, bootstrap, bandwidth, config get/set,
  shutdown (operations outside the CoreAPI iface list above).

Acceptance: an `exposure_matrix()` test referencing every symbol — the
crate fails to compile if any item regresses or stays unimplemented.
