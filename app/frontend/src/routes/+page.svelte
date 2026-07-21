<script lang="ts">
    import type { ExecuteRequest } from '$lib/types/ExecuteRequest';
    import type { ExecuteResponse } from '$lib/types/ExecuteResponse';
    import CodeMirror from "svelte-codemirror-editor";
    import { rust } from "@codemirror/lang-rust";
    import { catppuccinMocha } from "@catppuccin/codemirror";

    let value = $state("fn main() {\n    println!(\"Hello, World!\");\n}");
    let isExecuting = $state(false);
    let response: ExecuteResponse | null = $state(null);

    async function handleRun() {
      isExecuting = true;
      let payload: ExecuteRequest = {
          language: "rust",
          code: value,
      };
      const reply = await fetch('http://localhost:3000/api/v1/execute', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
      });
      if (!reply.ok) {
        console.error('Failed to execute code:', reply.statusText);
        isExecuting = false;
        return;
      }
      const data: ExecuteResponse = await reply.json();
      response = data;
      isExecuting = false;
    }
</script>

<main class="ide-container">
    <header class="toolbar">
        <div class="logo">
            <span class="bracket">[</span> ft_LGTM <span class="bracket">]</span>
        </div>
        <button class="run-btn {isExecuting ? 'pulsing' : ''}" onclick={handleRun} disabled={isExecuting}>
            {isExecuting ? 'EXECUTING...' : 'RUN CODE'}
        </button>
    </header>

    <div class="workspace">
        <section class="pane editor-pane">
            <div class="pane-header">Editor</div>
            <CodeMirror bind:value lang={rust()} theme={catppuccinMocha} />
        </section>

        <section class="pane output-pane">
            <div class="pane-header">Terminal</div>

            <div class="terminal-content">
                {#if isExecuting}
                    <div class="loader">
                        <span class="spinner"></span> Awaiting execution...
                    </div>
                {:else if response}
                    {#if response.status === "CompilationError"}
                        <div class="log-block error">
                            <span class="log-label">COMPILATION ERROR</span>
                            <pre>{response.compilation_log}</pre>
                            <pre>{response.stderr}</pre>
                        </div>
                    {:else if response.status === "RuntimeError"}
                        <div class="log-block error">
                            <span class="log-label">RUNTIME ERROR</span>
                            <pre>{response.stderr}</pre>
                        </div>
                        <div class="log-block stdout">
                            <span class="log-label">STDOUT</span>
                            <pre>{response.stdout}</pre>
                        </div>
                    {:else}
                        <div class="log-block compile">
                            <span class="log-label">BUILD</span>
                            <pre>{response.compilation_log}</pre>
                        </div>
                        <div class="log-block stdout">
                            <span class="log-label">STDOUT</span>
                            <pre>{response.stdout}</pre>
                        </div>

                        {#if response.ipfs_cid}
                            <div class="ipfs-link">
                                <span class="log-label">IPFS LINK</span>
                                <a href="https://ipfs.io/ipfs/{response.ipfs_cid}" target="_blank">
                                    ipfs.io/ipfs/{response.ipfs_cid}
                                </a>
                            </div>
                        {/if}
                    {/if}
                {:else}
                    <div class="idle-text">Ready.</div>
                {/if}
            </div>
        </section>
    </div>
</main>

<style>
    /* Layout */
    .ide-container {
        display: flex;
        flex-direction: column;
        height: 100vh;
        width: 100vw;
        overflow: hidden;
    }

    .toolbar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem 1.5rem;
        background-color: var(--bg-surface);
        border-bottom: 1px solid var(--border-subtle);
    }

    .workspace {
        display: grid;
        grid-template-columns: 1fr 1fr;
        flex: 1;
        overflow: hidden;
    }

    .pane {
        display: flex;
        flex-direction: column;
        height: 100%;
        overflow: hidden;
    }

    .editor-pane {
        border-right: 1px solid var(--border-subtle);
    }

    .pane-header {
        padding: 0.5rem 1rem;
        background-color: var(--bg-surface);
        color: var(--text-muted);
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        border-bottom: 1px solid var(--border-subtle);
    }

    /* Branding */
    .logo {
        font-weight: 700;
        font-size: 1.1rem;
        color: var(--text-main);
    }
    .logo .bracket {
        color: var(--accent-primary);
        font-weight: 300;
    }

    /* The Sleek Run Button */
    .run-btn {
        background-color: transparent;
        color: var(--accent-primary);
        border: 1px solid var(--accent-primary);
        padding: 0.5rem 1.5rem;
        font-family: var(--font-mono);
        font-weight: 700;
        font-size: 0.85rem;
        cursor: pointer;
        transition: all 0.2s ease-in-out;
        text-transform: uppercase;
        letter-spacing: 0.1em;
    }

    .run-btn:hover:not(:disabled) {
        background-color: var(--accent-glow);
        box-shadow: 0 0 10px var(--accent-glow);
    }

    .run-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .pulsing {
        animation: pulse 1.5s infinite alternate;
    }

    @keyframes pulse {
        0% { box-shadow: 0 0 0 0 var(--accent-glow); }
        100% { box-shadow: 0 0 15px 5px var(--accent-glow); }
    }

    /* Content Areas */
    .terminal-content {
        flex: 1;
        padding: 1rem;
        overflow-y: auto;
        background-color: var(--bg-base);
    }

    /* Terminal Outputs */
    pre {
        margin: 0;
        white-space: pre-wrap;
        word-break: break-all;
        font-size: 0.9rem;
        line-height: 1.5;
    }

    .log-block {
        margin-bottom: 1.5rem;
    }

    .log-label {
        display: block;
        font-size: 0.7rem;
        color: var(--text-muted);
        margin-bottom: 0.5rem;
        user-select: none;
    }

    .stdout pre { color: var(--text-main); }
    .compile pre { color: var(--text-muted); }
    .error pre { color: var(--accent-error); }

    .ipfs-link a {
        color: var(--accent-primary);
        text-decoration: none;
        word-break: break-all;
    }

    .ipfs-link a:hover {
        text-decoration: underline;
    }

    .idle-text {
        color: var(--text-muted);
        font-style: italic;
    }

    /* Hacker Spinner */
    .loader {
        display: flex;
        align-items: center;
        gap: 10px;
        color: var(--accent-primary);
    }

    .spinner {
        display: inline-block;
        width: 12px;
        height: 12px;
        background-color: var(--accent-primary);
        animation: blink 1s step-end infinite;
    }

    @keyframes blink {
        50% { opacity: 0; }
    }
</style>
