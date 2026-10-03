<script>
    let { isExecuting, response, ipfs_url } = $props();
</script>

<div class="pane-header">Terminal</div>
<div class="terminal-content">
    {#if isExecuting}
        <div class="loader">
            <span class="spinner"></span> Awaiting execution...
        </div>
    {:else if response}
        {#if response.status === "CompilationError"}
            <div class="log-block error">
                <span class="log-label">COMPILATION FAILED</span>
                <pre>{response.compilation_log}</pre>
            </div>
        {:else if response.status === "RuntimeError"}
            <div class="log-block error">
                <span class="log-label">RUNTIME ERROR</span>
                <pre>{response.stderr}</pre>
            </div>
            <details class="log-block stdout">
                <summary class="log-label">
                    {#if response.stdout}
                        <span class="icon">▶</span>
                    {/if}
                    STDOUT
                </summary>
                <pre>{response.stdout}</pre>
            </details>
        {:else}
            <details class="log-block compile">
                <summary class="log-label">
                    {#if response.compilation_log}
                        <span class="icon">▶</span>
                    {/if}
                    BUILD
                </summary>
                <pre>{response.compilation_log}</pre>
            </details>
            <div class="log-block runtime">
                <span class="log-label">EXECUTION TIME</span>
                <pre>{response.execution_time_ms}</pre>
            </div>
            <div class="log-block stdout">
                <span class="log-label">STDOUT</span>
                <pre>{response.stdout}</pre>
            </div>
            <div class="log-block error">
                <span class="log-label">STDERR</span>
                <pre>{response.stderr}</pre>
            </div>

            {#if response.ipfs_cid}
                <div class="ipfs-link">
                    <span class="log-label">IPFS LINK</span>
                    <a href="{ipfs_url}{response.ipfs_cid}" target="_blank">
                        {ipfs_url}{response.ipfs_cid}
                    </a>
                </div>
            {/if}
        {/if}
    {:else}
        <div class="idle-text">Ready.</div>
    {/if}
</div>

<style>
    .pane-header {
        padding: 0.5rem 1rem;
        background-color: var(--bg-surface);
        color: var(--text-muted);
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        border-bottom: 1px solid var(--border-subtle);
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

    details {
      user-select: none;
    }

    details>summary span.icon {
      width: 24px;
      height: 24px;
      transition: all 0.3s;
      margin-left: auto;
    }

    details[open] summary span.icon {
      transform: rotate(90deg);
    }

    summary {
      display: flex;
      cursor: pointer;
    }

    summary::-webkit-details-marker {
      display: none;
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
