<script lang="ts">
    import { PUBLIC_API_URL } from '$env/static/public';
    import { page } from '$app/state';
    import type { ExecuteResponse } from '$lib/types/ExecuteResponse';
    import Editor from '$lib/components/Editor.svelte';
	import Terminal from '$lib/components/Terminal.svelte';


    // TODO: fetch from backend after implementing the endpoint
    let code = "#include <unistd.h>\nint main()\n{\n\twrite(1, \"Hello, world!\", 13);\n}\n";
    let language = "c";
    let response: ExecuteResponse | null = $state(null);
</script>

<main class="ide-container">
    <header>
        <h1>Explore code snippets</h1>
    </header>
    <div>
        Snippet: {page.params.code}
    </div>

    <div class="workspace">
        <section class="pane editor-pane">
            <Editor bind:code {language} isExecuting={false} readonly=true />
        </section>

        <section class="pane output-pane">
            <Terminal isExecuting={false} {response} />
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

</style>
