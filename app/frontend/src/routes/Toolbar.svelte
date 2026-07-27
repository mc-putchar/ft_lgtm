<script lang="ts">
    let {
        language = $bindable(),
        isExecuting,
        onRun
    }: {
        language: string;
        isExecuting: boolean;
        onRun: () => void;
    } = $props();
</script>

<header class="toolbar">
    <div class="logo-and-controls">
        <div class="logo">
            <span class="bracket">[</span> ft_LGTM <span class="bracket">]</span>
        </div>

        <select class="language-selector" bind:value={language} disabled={isExecuting}>
            <option value="rust" selected>Rust</option>
            <option value="zig">Zig</option>
            <option value="c">C</option>
            <option value="cpp">C++</option>
        </select>
    </div>

    <button class="run-btn {isExecuting ? 'pulsing' : ''}" onclick={onRun} disabled={isExecuting}>
        {isExecuting ? 'EXECUTING...' : 'RUN CODE'}
    </button>
</header>

<style>
    .toolbar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.75rem 1.5rem;
        background-color: var(--bg-surface);
        border-bottom: 1px solid var(--border-subtle);
    }

    .logo-and-controls {
        display: flex;
        align-items: center;
        gap: 1.5rem;
    }

    .logo {
        font-weight: 700;
        font-size: 1.1rem;
        color: var(--text-main);
    }
    .logo .bracket {
        color: var(--accent-primary);
        font-weight: 300;
    }

    .language-selector {
        background-color: var(--bg-base);
        color: var(--text-main);
        border: 1px solid var(--border-subtle);
        padding: 0.4rem 0.8rem;
        font-family: var(--font-mono);
        font-size: 0.8rem;
        cursor: pointer;
        outline: none;
        transition: border-color 0.2s;
    }

    .language-selector:hover:not(:disabled) {
        border-color: var(--accent-primary);
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
</style>
