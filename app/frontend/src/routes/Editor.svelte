<script lang="ts">
    import CodeMirror from "svelte-codemirror-editor";
    import { rust } from "@codemirror/lang-rust";
    import { javascript } from "@codemirror/lang-javascript";
    import { python } from "@codemirror/lang-python";
    import { catppuccinMocha } from "@catppuccin/codemirror";

    let {
        code = $bindable(),
        language
    }: {
        code: string;
        language: string;
    } = $props();

    let languageExtension = $derived.by(() => {
        switch (language.toLowerCase()) {
            case "rust":
                return rust();
            case "javascript":
                return javascript();
            case "python":
                return python();
            default:
                return rust();
        }
    });
</script>

<section class="pane editor-pane">
    <div class="pane-header">Editor ({language})</div>
    <div class="editor-wrapper">
        <CodeMirror bind:value={code} lang={languageExtension} theme={catppuccinMocha} />
    </div>
</section>

<style>
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

    /* Ensures the CodeMirror instance fills the remaining pane height properly */
    .editor-wrapper {
        flex: 1;
        overflow: auto;
        display: flex;
        flex-direction: column;
    }

    .editor-wrapper :global(.cm-editor) {
        height: 100%;
    }
</style>
