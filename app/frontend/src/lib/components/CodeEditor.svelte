<script lang="ts">
    import CodeMirror from 'svelte-codemirror-editor';
    import { rust } from '@codemirror/lang-rust';
    import { cpp } from '@codemirror/lang-cpp';
    import { javascript } from '@codemirror/lang-javascript';
    import { go } from '@codemirror/lang-go';
    import { catppuccinLatte, catppuccinMacchiato, catppuccinMocha } from '@catppuccin/codemirror';

    type SupportedLanguage = 'rs' | 'c' | 'cpp' | 'go' | 'js';

    let {
        value = $bindable(''),
        language = 'rs',
        readonly = false,
        onchange = undefined,
        theme = 'mocha',
        class: className = ''
    }: {
        value?: string;
        language?: SupportedLanguage | string;
        readonly?: boolean;
        onchange?: (value: string) => void;
        theme?: string;
        class?: string;
    } = $props();

    let langExtension = $derived(() => {
        switch (language) {
            case 'rs':
                return rust();
            case 'c':
            case 'cpp':
                return cpp();
            case 'js':
                return javascript();
            case 'go':
                return go();
            default:
                return rust();
        }
    });

    function getCodeSample(lang: string | undefined) {
        switch (lang) {
            case 'rs':
                return "fn main() {\n\tprintln!(\"Hello, world!\");\n}";
            case 'c':
                return "#include <unistd.h>\n\nint main() {\n\twrite(STDOUT_FILENO, \"Hello, world!\\n\", 14);\n\treturn (0);\n}";
            case 'cpp':
                return "#include <iostream>\n\nint main() {\n\tstd::cout << \"Hello, world!\" << std::endl;\n\treturn 0;\n}"
            case 'go':
                return "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"Hello, world!\")\n}";
            case 'js':
                return "console.log(\"Hello, world!\");"
            default:
                return "fn main() {\n\tprintln!(\"Hello, world!\");\n}"
        }
    }

    $effect(() => {
        if (!readonly) {
            value = getCodeSample(language);
        }
    });

    let cmTheme = $derived(
        theme === 'latte'
            ? catppuccinLatte
            : theme === 'macchiato'
                ? catppuccinMacchiato
                : catppuccinMocha
    );
</script>

<div class={`${className}`}>
    <CodeMirror
        bind:value
        lang={langExtension()}
        theme={cmTheme}
        lineNumbers={true}
        tabSize={4}
        useTab={true}
        {readonly}
        {onchange}
        class="h-full w-full"
    />
</div>

<style>
    :global(.cm-editor) {
        height: 100% !important;
        background-color: transparent !important;
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace !important;
    }
    :global(.cm-scroller) {
        overflow: auto !important;
        font-family: inherit !important;
    }
    :global(.cm-gutters) {
        background-color: var(--cm-gutter-bg, #070a10) !important;
        border-right: 1px solid var(--cm-gutter-border, #1f293d) !important;
        color: #4b5563 !important;
    }
</style>
