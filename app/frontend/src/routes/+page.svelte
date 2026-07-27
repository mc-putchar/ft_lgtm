<script lang="ts">
    import type { ExecuteRequest } from '$lib/types/ExecuteRequest';
    import type { ExecuteResponse } from '$lib/types/ExecuteResponse';
    import Toolbar from './Toolbar.svelte';
	import Editor from './Editor.svelte';
	import Terminal from './Terminal.svelte';

    let language = $state("rust");
    let code = $state("fn main() {\n    println!(\"Hello, World!\");\n}");
    let isExecuting = $state(false);
    let response: ExecuteResponse | null = $state(null);

    async function handleRun() {
      isExecuting = true;
      let payload: ExecuteRequest = {
          language,
          code,
      };
      try {
        await executeCode(payload);
      } catch (error) {
        console.error('Error executing code:', error);
        response = {
          status: "CompilationError",
          stdout: "",
          stderr: "Error executing code: " + error,
          compilation_log: "",
          execution_time_ms: BigInt(0),
          ipfs_cid: "",
        };
      } finally {
        isExecuting = false;
      }
    }

    async function executeCode(payload: ExecuteRequest) {
      const reply = await fetch('/api/v1/execute', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
      });
      if (!reply.ok) {
        console.error('Failed to execute code:', reply.statusText);
        return;
      }
      const data: ExecuteResponse = await reply.json();
      response = data;
    }
</script>

<main class="ide-container">

    <Toolbar bind:language {isExecuting} onRun={handleRun} />

    <div class="workspace">
        <section class="pane editor-pane">
            <Editor bind:code {language} />
        </section>

        <section class="pane output-pane">
            <Terminal {isExecuting} {response} />
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
