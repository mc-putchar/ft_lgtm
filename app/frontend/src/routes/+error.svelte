<script lang="ts">
	import { page } from '$app/state';

	const emojis = {
		420: '🫠',
		500: '💥'
	};
</script>

<h1>{page.status} {page.error.message}</h1>
<span style="font-size: 10em">
	{emojis[page.status] ?? emojis[500]}
</span>
