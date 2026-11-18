<script lang="ts">
	let { class: className = "size-4" }: { class?: string } = $props();
</script>

<svg
	xmlns="http://www.w3.org/2000/svg"
	viewBox="0 0 32 32"
	fill="none"
	class={className}
	aria-hidden="true"
>
	<circle cx="16" cy="8" r="3" fill="currentColor" />
	<circle cx="9" cy="22" r="3" fill="currentColor" />
	<circle cx="23" cy="22" r="3" fill="currentColor" />
	<line x1="16" y1="8" x2="9" y2="22" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
	<line x1="16" y1="8" x2="23" y2="22" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
	<line
		x1="9"
		y1="22"
		x2="23"
		y2="22"
		stroke="currentColor"
		stroke-width="2"
		stroke-dasharray="2 2"
		stroke-linecap="round"
	/>
</svg>
