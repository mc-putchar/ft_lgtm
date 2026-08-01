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
    <select class="language-selector" bind:value={language} disabled={isExecuting}>
        <option value="rust" selected>Rust</option>
        <option value="c">C</option>
        <option value="cpp" disabled>C++</option>
        <option value="zig" disabled>Zig</option>
    </select>
    <button class="run-btn {isExecuting ? 'pulsing' : ''}" onclick={onRun} disabled={isExecuting}>
        {isExecuting ? 'EXECUTING...' : '▶ RUN CODE'}
    </button>
</header>

<style>
    .toolbar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.5rem 1rem;
    }

    .language-selector {
        background-color: var(--bg-base);
        color: var(--text-main);
        border: 1px solid var(--border-subtle);
        padding: 0.2rem 0.2rem;
        font-family: var(--font-mono);
        font-size: 0.8rem;
        cursor: pointer;
        outline: none;
        transition: border-color 0.2s;
    }

    .language-selector:hover:not(:disabled) {
        border-color: var(--accent-primary);
    }

    .run-btn {
        background-color: transparent;
        color: var(--accent-primary);
        border: 1px solid var(--accent-primary);
        padding: 0.5rem 1rem;
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
</style>
