# TODO — pending upstream fixes

## SuperSwinkAI/Swink-Agent#572 — TUI ignores `ToolExecutionUpdate` events

Once the TUI renders `ToolExecutionUpdate` events in the tool panel, the three
pipeline tools (`orchestrate_research`, `research_question`, `synthesize_and_save`)
can use their `on_update` callbacks to stream phase-level progress rather than
showing a silent spinner until completion.

## SuperSwinkAI/Swink-Agent#573 — Built-in `InMemoryArtifactStore` and `FsArtifactStore`

`InMemoryArtifactStore` in `src/main.rs` is ~60 lines of boilerplate that
exists solely because the crate ships no concrete store. Once upstream provides
one, delete the local `InMemoryArtifactStore` impl and replace with:

```rust
use swink_agent::artifact::InMemoryArtifactStore;
```

The `FsArtifactStore` would also allow research artifacts to persist across
runs — worth considering as an option for the synthesize step.

## SuperSwinkAI/Swink-Agent#574 — `FnTool::with_execute_typed`

The three param structs (`OrchestrateParams`, `ResearchQuestionParams`,
`SynthesizeParams`) use `#[allow(dead_code)]` because fields are declared for
schema generation but values are extracted via raw `params["field"].as_str()`.

Once `with_execute_typed` ships, rewrite each tool builder to use it:

```rust
FnTool::new(...)
    .with_execute_typed(|p: OrchestrateParams, _cancel| async move {
        // use p.topic directly — no raw JSON extraction, no allow(dead_code)
    })
```

Remove the `#[allow(dead_code)]` annotations and the raw extraction calls at
the same time.
