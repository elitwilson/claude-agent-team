## Decision: scheduler module stub for tests
Date: 2026-03-30
Context: Spec says main.rs should call scheduler::schedule_run, but no scheduler module exists yet (spec 005). Tests for TuiResult extension need to verify the scheduled_at field exists on TuiResult, but cannot test the actual scheduler call.
Assumption: Tests will verify TuiResult.scheduled_at field presence and that App::result() returns None for scheduled_at in the immediate-run path. The actual scheduler::schedule_run wiring in main.rs will be tested when spec 005 provides the module.
