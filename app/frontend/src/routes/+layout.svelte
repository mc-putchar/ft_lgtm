<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '$lib/assets/style.css';

	import { page } from '$app/state';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<header class="navbar">
    <div class="logo-and-controls">
        <div class="logo">
            <span class="bracket pulsing">[</span>
            <a href="/" aria-current={page.url.pathname === '/'}>
                ft_LGTM
            </a>
            <span class="bracket pulsing">]</span>
        </div>
        <a href="/x" aria-current={page.url.pathname === '/x'}>Explore</a>
        <a href="/exec" aria-current={page.url.pathname === '/exec'}>Playground</a>
    </div>
</header>

{@render children()}
