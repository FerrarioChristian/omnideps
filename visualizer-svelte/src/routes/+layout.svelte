<script>
    import '../app.css';
    import { page } from '$app/stores';

    let { children } = $props();
</script>

<div class="nav-menu">
    <a href="/" class:active={$page.url.pathname === '/'}>Benchmarks</a>
    <a href="/reports" class:active={$page.url.pathname.startsWith('/reports')}>Reports</a>
    <a href="/custom" class:active={$page.url.pathname.startsWith('/custom')}>Custom Analysis</a>
    <a href="/docs" class:active={$page.url.pathname.startsWith('/docs')}>Docs</a>
</div>

{@render children()}
