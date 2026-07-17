"""Run trace replay and capture stdout/stderr properly."""
import sys
import trace_replay

results = trace_replay.replay_all(csv_name='traces_lite.csv')
print(f'\n=== REPLAY RESULTS ({len(results)} traces) ===', flush=True)
for r in results:
    print(f'\n[{r.trace_id[:8]}] agent={r.agent} | role={r.inferred_role}', flush=True)
    print(f'  forge_success={r.forge_success} latency={r.forge_latency_s:.1f}s out_len={r.forge_output_length}', flush=True)
    print(f'  intent_match={r.intent_match:.3f} tool_seq_match={r.tool_sequence_match:.3f}', flush=True)
    print(f'  subagent_match={r.subagent_topology_match:.3f} output_sim={r.output_similarity:.3f}', flush=True)
    print(f'  overall_replay_score={r.overall_replay_score:.3f}', flush=True)
    if r.judge_feedback:
        print(f'  feedback: {r.judge_feedback[:200]}', flush=True)
    if r.judge_error:
        print(f'  judge_error: {r.judge_error[:200]}', flush=True)
    if r.forge_error:
        print(f'  forge_error: {r.forge_error[:200]}', flush=True)

out = trace_replay.write_results(results, 'replay_lite_results.json')
print(f'\nResults saved to: {out}', flush=True)