<script lang="ts">
	import { Button } from "$lib/components/ui/button/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import DarkModeSwitcher from "$lib/components/theme/dark-mode-switcher.svelte";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";

	// Lucide Icons
	import ArrowRightIcon from "@lucide/svelte/icons/arrow-right";
	import BookOpenIcon from "@lucide/svelte/icons/book-open";
	import TerminalIcon from "@lucide/svelte/icons/terminal";
	import DownloadIcon from "@lucide/svelte/icons/download";
	import CheckIcon from "@lucide/svelte/icons/check";
	import CopyIcon from "@lucide/svelte/icons/copy";
	import GithubIcon from "@lucide/svelte/icons/github";
	import LayersIcon from "@lucide/svelte/icons/layers";
	import BoxIcon from "@lucide/svelte/icons/box";
	import GlobeIcon from "@lucide/svelte/icons/globe";
	import CpuIcon from "@lucide/svelte/icons/cpu";
	import GitBranchIcon from "@lucide/svelte/icons/git-branch";
	import SparklesIcon from "@lucide/svelte/icons/sparkles";
	import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
	import CodeIcon from "@lucide/svelte/icons/code";
	import MonitorIcon from "@lucide/svelte/icons/monitor";
	import FileTextIcon from "@lucide/svelte/icons/file-text";
	import RefreshCwIcon from "@lucide/svelte/icons/refresh-cw";
	import { fetchLatestRelease, FALLBACK_VERSION, FALLBACK_RELEASE_URL } from "$lib/docs/version.js";

	// State for copy buttons and preview tabs
	let copiedCommand = $state(false);
	let copiedJson = $state(false);
	let activeTab = $state<"terminal" | "visualizer" | "json">("visualizer");
	let selectedOs = $state<"mac" | "linux" | "windows">("mac");
	let latestVersion = $state(FALLBACK_VERSION);
	let releaseNotesUrl = $state(FALLBACK_RELEASE_URL);

	// Interactive Terminal State
	let activeCliCommand = $state<"analyze" | "serve" | "benchmark">("analyze");

	// Interactive Visualizer State
	type NodeType = "class" | "trait" | "function" | "service";
	interface GraphNode {
		id: string;
		name: string;
		type: NodeType;
		lang: "Java" | "Rust" | "Python" | "C";
		package: string;
		file: string;
		line: number;
		x: number;
		y: number;
		hx: number;
		hy: number;
	}

	interface GraphEdge {
		source: string;
		target: string;
		kind: "invokes" | "implements" | "field" | "imports";
		langRelation: string;
	}

	const demoNodes: GraphNode[] = [
		{
			id: "main",
			name: "main()",
			type: "function",
			lang: "C",
			package: "native::core",
			file: "src/main.c",
			line: 18,
			x: 100,
			y: 110,
			hx: 80,
			hy: 110
		},
		{
			id: "AppRouter",
			name: "AppRouter",
			type: "class",
			lang: "Rust",
			package: "crate::server",
			file: "src/server/router.rs",
			line: 34,
			x: 230,
			y: 50,
			hx: 220,
			hy: 50
		},
		{
			id: "AuthService",
			name: "AuthService",
			type: "service",
			lang: "Python",
			package: "app.services.auth",
			file: "services/auth.py",
			line: 45,
			x: 230,
			y: 170,
			hx: 220,
			hy: 170
		},
		{
			id: "DatabaseTrait",
			name: "DatabaseService",
			type: "trait",
			lang: "Java",
			package: "com.omnideps.db",
			file: "src/db/DatabaseService.java",
			line: 12,
			x: 390,
			y: 110,
			hx: 380,
			hy: 110
		},
		{
			id: "PgPool",
			name: "PostgresImpl",
			type: "class",
			lang: "Java",
			package: "com.omnideps.db.pg",
			file: "src/db/PostgresImpl.java",
			line: 28,
			x: 520,
			y: 60,
			hx: 510,
			hy: 60
		},
		{
			id: "RedisClient",
			name: "RedisCache",
			type: "class",
			lang: "Rust",
			package: "crate::cache",
			file: "src/cache/redis.rs",
			line: 22,
			x: 520,
			y: 160,
			hx: 510,
			hy: 160
		}
	];

	const demoEdges: GraphEdge[] = [
		{ source: "main", target: "AppRouter", kind: "invokes", langRelation: "C → Rust FFI" },
		{ source: "main", target: "AuthService", kind: "invokes", langRelation: "C → Python API" },
		{ source: "AppRouter", target: "DatabaseTrait", kind: "field", langRelation: "Rust → Java Bridge" },
		{ source: "AuthService", target: "DatabaseTrait", kind: "invokes", langRelation: "Python → Java RPC" },
		{ source: "DatabaseTrait", target: "PgPool", kind: "implements", langRelation: "Java Interface" },
		{ source: "DatabaseTrait", target: "RedisClient", kind: "imports", langRelation: "Java → Rust Cache" }
	];

	let selectedNodeId = $state<string>("DatabaseTrait");
	let filterType = $state<"all" | "classes" | "traits" | "functions">("all");
	let layoutMode = $state<"fCoSE" | "hierarchical">("fCoSE");

	let selectedNode = $derived(demoNodes.find((n) => n.id === selectedNodeId) ?? demoNodes[3]);

	let incomingEdges = $derived(demoEdges.filter((e) => e.target === selectedNode.id));
	let outgoingEdges = $derived(demoEdges.filter((e) => e.source === selectedNode.id));

	let visibleNodes = $derived(
		demoNodes.filter((node) => {
			if (filterType === "all") return true;
			if (filterType === "classes") return node.type === "class" || node.type === "service";
			if (filterType === "traits") return node.type === "trait";
			if (filterType === "functions") return node.type === "function";
			return true;
		})
	);

	function isNodeVisible(id: string): boolean {
		return visibleNodes.some((n) => n.id === id);
	}

	function isEdgeConnected(e: GraphEdge): boolean {
		return e.source === selectedNodeId || e.target === selectedNodeId;
	}

	// Schema switcher
	let schemaMode = $state<"canonical" | "cytoscape">("canonical");

	$effect(() => {
		if (typeof window !== "undefined") {
			const ua = navigator.userAgent.toLowerCase();
			if (ua.includes("win")) selectedOs = "windows";
			else if (ua.includes("linux")) selectedOs = "linux";
			else selectedOs = "mac";

			fetchLatestRelease().then((info) => {
				latestVersion = info.tag;
				releaseNotesUrl = info.url;
			});
		}
	});

	const releaseBaseUrl = "https://github.com/FerrarioChristian/omnideps/releases/latest/download";

	const platformDownloads = {
		macSilicon: `${releaseBaseUrl}/omnideps-macos-aarch64`,
		macIntel: `${releaseBaseUrl}/omnideps-macos-x86_64`,
		linux: `${releaseBaseUrl}/omnideps-linux-x86_64`,
		windows: `${releaseBaseUrl}/omnideps-windows-x86_64.exe`
	};

	let primaryDownloadUrl = $derived(
		selectedOs === "windows"
			? platformDownloads.windows
			: selectedOs === "linux"
				? platformDownloads.linux
				: platformDownloads.macSilicon
	);

	function copyToClipboard(text: string) {
		if (typeof navigator !== "undefined") {
			navigator.clipboard.writeText(text);
			copiedCommand = true;
			setTimeout(() => {
				copiedCommand = false;
			}, 2000);
		}
	}

	function copyJsonToClipboard(text: string) {
		if (typeof navigator !== "undefined") {
			navigator.clipboard.writeText(text);
			copiedJson = true;
			setTimeout(() => {
				copiedJson = false;
			}, 2000);
		}
	}

	const installCommand = "curl -fsSL https://raw.githubusercontent.com/FerrarioChristian/omnideps/main/install.sh | bash";

	const features = [
		{
			icon: GlobeIcon,
			title: "Unified Polyglot Graph",
			description:
				"Scans multiple languages simultaneously (Java, Rust, Python, C, C++) and connects all entities into one coherent, unified architectural graph."
		},
		{
			icon: BoxIcon,
			title: "Single Standalone Binary",
			description:
				"Pre-compiled Rust executable with zero runtime dependencies. No Node.js, Python, or JVM required on the target machine."
		},
		{
			icon: MonitorIcon,
			title: "Embedded Web Visualizer",
			description:
				"Run `omnideps serve` to launch a responsive Cytoscape web UI directly in your browser with physics-driven graph layouts."
		},
		{
			icon: CpuIcon,
			title: "Tree-sitter & Rayon Core",
			description:
				"High-speed AST parsing powered by official Tree-sitter grammars, parallelized across CPU threads for sub-second analysis."
		},
		{
			icon: GitBranchIcon,
			title: "Deep Type & Name Resolution",
			description:
				"Traces cross-file type references, class and interface inheritance, trait implementations, and parameterized generics."
		},
		{
			icon: FileTextIcon,
			title: "Rich Exports & Benchmarks",
			description:
				"Export to Cytoscape JSON, raw JSON, and CSV metrics. Includes a built-in benchmarking harness to verify AST extraction accuracy."
		}
	];

	// Java is first and Stable. All others are marked Experimental.
	const languages = [
		{
			name: "Java",
			grammar: "tree-sitter-java",
			entities: "Packages, Classes, Interfaces, Methods, Fields",
			features: "Package hierarchies, inheritance chains, wildcard imports, generics",
			status: "Stable"
		},
		{
			name: "Rust",
			grammar: "tree-sitter-rust",
			entities: "Modules, Structs, Traits, Impls, Functions",
			features: "Crate & mod path resolution, generic bounds, trait dispatch",
			status: "Experimental"
		},
		{
			name: "Python",
			grammar: "tree-sitter-python",
			entities: "Modules, Classes, Functions, Decorators",
			features: "Absolute & relative imports, dynamic scope fallback",
			status: "Experimental"
		},
		{
			name: "C++",
			grammar: "tree-sitter-cpp",
			entities: "Namespaces, Classes, Structs, Functions, Headers",
			features: "Scope resolution (::), includes, template structures",
			status: "Experimental"
		},
		{
			name: "C",
			grammar: "tree-sitter-c",
			entities: "Header units, Structs, Functions, Typedefs",
			features: "Header graph linking, function signatures, typedefs",
			status: "Experimental"
		}
	];
</script>

<svelte:head>
	<title>Omnideps • Language-Agnostic Architectural Dependency Analyzer</title>
	<meta
		name="description"
		content="A language-agnostic architectural dependency analyzer and visualizer based on Tree-sitter and Rust. Unifies polyglot codebases into a single graph."
	/>
</svelte:head>

<!-- Top Navigation Bar -->
<header class="bg-background/80 border-border sticky top-0 z-50 w-full border-b backdrop-blur-md">
	<div class="mx-auto flex h-14 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
		<!-- Brand Logo -->
		<a href="/" class="flex items-center gap-2.5 transition-opacity hover:opacity-90">
			<div class="bg-foreground text-background flex size-7 items-center justify-center rounded-md font-mono text-sm font-bold">
				⬡
			</div>
			<div class="flex items-center gap-2">
				<span class="text-base font-semibold tracking-tight">omnideps</span>
				<Badge variant="outline" class="font-mono text-[10px] px-1.5 py-0 h-4">{latestVersion}</Badge>
			</div>
		</a>

		<!-- Desktop Nav Links -->
		<nav class="hidden items-center gap-6 text-sm font-medium md:flex">
			<a href="/docs" class="text-muted-foreground hover:text-foreground transition-colors">Documentation</a>
			<a href="/docs/cli/analyze" class="text-muted-foreground hover:text-foreground transition-colors">CLI</a>
			<a href="/docs/architecture/languages" class="text-muted-foreground hover:text-foreground transition-colors">Languages</a>
			<a href="#polyglot" class="text-muted-foreground hover:text-foreground transition-colors">Polyglot Engine</a>
			<a href="#features" class="text-muted-foreground hover:text-foreground transition-colors">Features</a>
			<a
				href="https://github.com/FerrarioChristian/omnideps/releases"
				target="_blank"
				class="text-muted-foreground hover:text-foreground transition-colors"
			>
				Releases
			</a>
		</nav>

		<!-- Right Action Buttons -->
		<div class="flex items-center gap-2">
			<DarkModeSwitcher />
			<Button
				variant="ghost"
				size="icon"
				href="https://github.com/FerrarioChristian/omnideps"
				target="_blank"
				aria-label="GitHub Repository"
			>
				<GithubIcon class="size-4" />
			</Button>
			<Button size="sm" href="/docs/getting-started/quickstart" class="hidden sm:inline-flex gap-1.5">
				Get Started
				<ArrowRightIcon class="size-3.5" />
			</Button>
		</div>
	</div>
</header>

<main class="flex-1">
	<!-- Hero Section -->
	<section class="relative overflow-hidden pt-16 pb-20 sm:pt-24 sm:pb-32 border-b">
		<div class="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_80%_80%_at_50%_-20%,rgba(120,119,198,0.1),rgba(255,255,255,0))] dark:bg-[radial-gradient(ellipse_80%_80%_at_50%_-20%,rgba(120,119,198,0.15),rgba(0,0,0,0))]"></div>

		<div class="relative mx-auto max-w-5xl px-4 text-center sm:px-6 lg:px-8">
			<!-- Announcement Pill -->
			<a
				href={releaseNotesUrl}
				target="_blank"
				class="border-border bg-muted/60 hover:bg-muted text-muted-foreground hover:text-foreground mb-8 inline-flex items-center gap-2 rounded-full border px-3.5 py-1 text-xs font-medium transition-colors"
			>
				<SparklesIcon class="size-3.5 text-foreground" />
				<span>Omnideps {latestVersion} Released</span>
				<span class="text-muted-foreground/40">•</span>
				<span class="inline-flex items-center gap-1 font-normal">
					View Release Notes
					<ArrowRightIcon class="size-3" />
				</span>
			</a>

			<!-- Main Heading -->
			<h1 class="text-foreground text-4xl font-extrabold tracking-tight sm:text-6xl lg:text-7xl">
				Architectural Dependency Analysis
				<span class="block text-muted-foreground font-normal sm:mt-1">
					at the Speed of Rust.
				</span>
			</h1>

			<!-- Subtitle -->
			<p class="text-muted-foreground mx-auto mt-6 max-w-3xl text-lg sm:text-xl font-normal leading-relaxed">
				Extract, resolve, and interactively explore code dependency graphs across
				<strong>Java, Rust, Python, C, and C++</strong>. Scans polyglot projects together into a <strong>single unified graph</strong>. Standalone binary with an embedded web visualizer.
			</p>

			<!-- Action Buttons & Download Dropdown -->
			<div class="mt-10 flex flex-wrap items-center justify-center gap-4">
				<div class="inline-flex rounded-md shadow-xs">
					<Button
						size="lg"
						href={primaryDownloadUrl}
						class="gap-2 px-6 rounded-r-none border-r border-primary-foreground/20"
					>
						<DownloadIcon class="size-4" />
						Download for {selectedOs === "mac" ? "macOS" : selectedOs === "windows" ? "Windows" : "Linux"}
					</Button>
					<DropdownMenu.Root>
						<DropdownMenu.Trigger>
							{#snippet child({ props })}
								<Button size="lg" class="px-2.5 rounded-l-none" {...props} aria-label="Choose platform">
									<ChevronDownIcon class="size-4" />
								</Button>
							{/snippet}
						</DropdownMenu.Trigger>
						<DropdownMenu.Content align="end" class="w-64">
							<DropdownMenu.Label>Direct Binary Downloads</DropdownMenu.Label>
							<DropdownMenu.Separator />
							<DropdownMenu.Item>
								<a
									href={platformDownloads.macSilicon}
									class="flex w-full items-center justify-between"
								>
									<span>macOS (Apple Silicon)</span>
									<span class="text-muted-foreground text-xs font-mono">arm64</span>
								</a>
							</DropdownMenu.Item>
							<DropdownMenu.Item>
								<a
									href={platformDownloads.macIntel}
									class="flex w-full items-center justify-between"
								>
									<span>macOS (Intel)</span>
									<span class="text-muted-foreground text-xs font-mono">x86_64</span>
								</a>
							</DropdownMenu.Item>
							<DropdownMenu.Item>
								<a
									href={platformDownloads.linux}
									class="flex w-full items-center justify-between"
								>
									<span>Linux</span>
									<span class="text-muted-foreground text-xs font-mono">x86_64</span>
								</a>
							</DropdownMenu.Item>
							<DropdownMenu.Item>
								<a
									href={platformDownloads.windows}
									class="flex w-full items-center justify-between"
								>
									<span>Windows</span>
									<span class="text-muted-foreground text-xs font-mono">.exe</span>
								</a>
							</DropdownMenu.Item>
							<DropdownMenu.Separator />
							<DropdownMenu.Item>
								<a
									href="https://github.com/FerrarioChristian/omnideps/releases/latest"
									target="_blank"
									class="text-xs text-muted-foreground flex items-center justify-between w-full"
								>
									<span>View all releases on GitHub</span>
									<span>↗</span>
								</a>
							</DropdownMenu.Item>
							<DropdownMenu.Item>
								<a href="/docs/getting-started/installation" class="text-xs text-muted-foreground">
									Build from source instructions →
								</a>
							</DropdownMenu.Item>
						</DropdownMenu.Content>
					</DropdownMenu.Root>
				</div>

				<Button variant="outline" size="lg" href="/docs" class="gap-2 px-6">
					<BookOpenIcon class="size-4" />
					Documentation
				</Button>

				<Button
					variant="ghost"
					size="lg"
					href="https://github.com/FerrarioChristian/omnideps"
					target="_blank"
					class="gap-2 px-5 text-muted-foreground hover:text-foreground"
				>
					<GithubIcon class="size-4" />
					GitHub
				</Button>
			</div>

			<!-- Quick Install Terminal Bar (Wider box so everything comfortably fits on one line) -->
			<div class="mt-8 mx-auto w-full max-w-3xl">
				<div class="border-border bg-muted/40 flex items-center justify-between rounded-lg border px-4 py-2.5 font-mono text-xs sm:text-sm shadow-xs">
					<div class="flex items-center gap-2 overflow-x-auto text-left whitespace-nowrap scrollbar-none">
						<span class="text-muted-foreground select-none">$</span>
						<span class="text-foreground tracking-tight">{installCommand}</span>
					</div>
					<button
						type="button"
						onclick={() => copyToClipboard(installCommand)}
						class="text-muted-foreground hover:text-foreground ml-3 inline-flex size-8 shrink-0 items-center justify-center rounded-md border border-transparent hover:border-border hover:bg-background transition-all cursor-pointer"
						title="Copy install command"
					>
						{#if copiedCommand}
							<CheckIcon class="size-4 text-emerald-500" />
						{:else}
							<CopyIcon class="size-4" />
						{/if}
					</button>
				</div>
			</div>
		</div>

		<!-- Fully Interactive macOS Showcase Window -->
		<div class="relative z-10 mx-auto mt-14 max-w-5xl px-4 sm:px-6 lg:px-8">
			<div class="border-border bg-card text-card-foreground overflow-hidden rounded-xl border shadow-2xl">
				<!-- Window Header Bar with interactive controls and mode tabs -->
				<div class="border-border bg-muted/60 relative z-20 flex flex-wrap items-center justify-between gap-3 border-b px-4 py-2.5">
					<div class="flex items-center gap-2">
						<span class="size-3 rounded-full bg-red-500/80 transition-opacity hover:opacity-100"></span>
						<span class="size-3 rounded-full bg-yellow-500/80 transition-opacity hover:opacity-100"></span>
						<span class="size-3 rounded-full bg-green-500/80 transition-opacity hover:opacity-100"></span>
						<span class="ml-2 font-mono text-xs text-muted-foreground hidden sm:inline-block">
							omnideps — interactive live preview
						</span>
					</div>

					<!-- View switcher tabs -->
					<div class="flex items-center gap-1 rounded-lg border border-border bg-background/80 p-1 text-xs">
						<button
							type="button"
							onclick={() => (activeTab = "visualizer")}
							class={`flex items-center gap-1.5 rounded-md px-3 py-1 font-medium transition-all cursor-pointer ${
								activeTab === "visualizer"
									? "bg-foreground text-background shadow-xs"
									: "text-muted-foreground hover:text-foreground"
							}`}
						>
							<MonitorIcon class="size-3" />
							Web Visualizer
						</button>
						<button
							type="button"
							onclick={() => (activeTab = "terminal")}
							class={`flex items-center gap-1.5 rounded-md px-3 py-1 font-medium transition-all cursor-pointer ${
								activeTab === "terminal"
									? "bg-foreground text-background shadow-xs"
									: "text-muted-foreground hover:text-foreground"
							}`}
						>
							<TerminalIcon class="size-3" />
							CLI Terminal
						</button>
						<button
							type="button"
							onclick={() => (activeTab = "json")}
							class={`flex items-center gap-1.5 rounded-md px-3 py-1 font-medium transition-all cursor-pointer ${
								activeTab === "json"
									? "bg-foreground text-background shadow-xs"
									: "text-muted-foreground hover:text-foreground"
							}`}
						>
							<CodeIcon class="size-3" />
							Unified IR
						</button>
					</div>
				</div>

				<!-- Window Body Content -->
				<div class="bg-zinc-950 p-5 text-xs text-zinc-300 sm:text-sm min-h-[360px] flex flex-col justify-between">
					{#if activeTab === "visualizer"}
						<!-- 1. FULLY INTERACTIVE VISUALIZER -->
						<div class="relative w-full h-[340px] rounded-lg border border-zinc-800 bg-zinc-900/90 p-4 flex flex-col justify-between overflow-hidden select-none">
							<!-- Top Interactive Filter Bar -->
							<div class="flex flex-wrap items-center justify-between gap-2 border-b border-zinc-800 pb-2.5">
								<div class="flex items-center gap-1.5">
									<span class="text-[11px] text-zinc-400 font-medium mr-1">Filter Type:</span>
									<button
										type="button"
										onclick={() => (filterType = "all")}
										class={`rounded px-2 py-0.5 text-[10px] font-mono transition-colors cursor-pointer ${
											filterType === "all" ? "bg-white text-zinc-900 font-bold" : "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
										}`}
									>
										All ({demoNodes.length})
									</button>
									<button
										type="button"
										onclick={() => (filterType = "traits")}
										class={`rounded px-2 py-0.5 text-[10px] font-mono transition-colors cursor-pointer ${
											filterType === "traits" ? "bg-emerald-400 text-zinc-950 font-bold" : "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
										}`}
									>
										Interfaces/Traits
									</button>
									<button
										type="button"
										onclick={() => (filterType = "classes")}
										class={`rounded px-2 py-0.5 text-[10px] font-mono transition-colors cursor-pointer ${
											filterType === "classes" ? "bg-blue-400 text-zinc-950 font-bold" : "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
										}`}
									>
										Classes & Structs
									</button>
									<button
										type="button"
										onclick={() => (filterType = "functions")}
										class={`rounded px-2 py-0.5 text-[10px] font-mono transition-colors cursor-pointer ${
											filterType === "functions" ? "bg-amber-400 text-zinc-950 font-bold" : "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
										}`}
									>
										Functions
									</button>
								</div>

								<!-- Layout & State Controls -->
								<div class="flex items-center gap-2">
									<button
										type="button"
										onclick={() => (layoutMode = layoutMode === "fCoSE" ? "hierarchical" : "fCoSE")}
										class="flex items-center gap-1 rounded bg-zinc-800 hover:bg-zinc-700 px-2 py-0.5 text-[10px] font-mono text-zinc-300 transition-colors cursor-pointer"
										title="Toggle Layout simulation"
									>
										<RefreshCwIcon class="size-2.5" />
										Layout: {layoutMode}
									</button>
									<span class="text-[10px] text-zinc-500 font-mono hidden md:inline">Click any node to inspect</span>
								</div>
							</div>

							<!-- SVG Graph Canvas with Clickable Nodes & Incident Edge Highlighting -->
							<div class="relative flex-1 flex items-center justify-center my-2">
								<svg class="w-full h-full max-h-[220px]" viewBox="0 0 620 220" fill="none" xmlns="http://www.w3.org/2000/svg">
									<defs>
										<marker id="arrow" viewBox="0 0 10 10" refX="18" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
											<path d="M 0 0 L 10 5 L 0 10 z" fill="#71717a" />
										</marker>
										<marker id="arrow-active" viewBox="0 0 10 10" refX="18" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
											<path d="M 0 0 L 10 5 L 0 10 z" fill="#10b981" />
										</marker>
									</defs>

									<!-- Edges -->
									{#each demoEdges as edge}
										{@const sourceNode = demoNodes.find((n) => n.id === edge.source)}
										{@const targetNode = demoNodes.find((n) => n.id === edge.target)}
										{#if sourceNode && targetNode && isNodeVisible(sourceNode.id) && isNodeVisible(targetNode.id)}
											{@const active = isEdgeConnected(edge)}
											{@const sx = layoutMode === "fCoSE" ? sourceNode.x : sourceNode.hx}
											{@const sy = layoutMode === "fCoSE" ? sourceNode.y : sourceNode.hy}
											{@const tx = layoutMode === "fCoSE" ? targetNode.x : targetNode.hx}
											{@const ty = layoutMode === "fCoSE" ? targetNode.y : targetNode.hy}

											<line
												x1={sx}
												y1={sy}
												x2={tx}
												y2={ty}
												stroke={active ? "#10b981" : "#3f3f46"}
												stroke-width={active ? "2.2" : "1.2"}
												stroke-dasharray={edge.kind === "imports" ? "4 4" : undefined}
												marker-end={active ? "url(#arrow-active)" : "url(#arrow)"}
												class="transition-all duration-300"
											/>
										{/if}
									{/each}

									<!-- Nodes -->
									{#each demoNodes as node}
										{#if isNodeVisible(node.id)}
											{@const isSelected = selectedNodeId === node.id}
											{@const nx = layoutMode === "fCoSE" ? node.x : node.hx}
											{@const ny = layoutMode === "fCoSE" ? node.y : node.hy}

											<!-- svelte-ignore a11y_click_events_have_key_events -->
											<g
												role="button"
												tabindex="0"
												onclick={() => (selectedNodeId = node.id)}
												class="cursor-pointer group focus:outline-none"
											>
												{#if node.type === "function"}
													<!-- Function: Circle Node -->
													<circle
														cx={nx}
														cy={ny}
														r={isSelected ? 26 : 22}
														fill={isSelected ? "#27272a" : "#18181b"}
														stroke={isSelected ? "#f59e0b" : "#71717a"}
														stroke-width={isSelected ? "3" : "1.8"}
														class="transition-all duration-200 group-hover:stroke-white"
													/>
													<text x={nx} y={ny + 4} text-anchor="middle" fill="#f4f4f5" font-size="9" font-family="monospace">
														{node.name}
													</text>
												{:else}
													<!-- Class/Trait/Service: Rect Node -->
													<rect
														x={nx - 48}
														y={ny - 18}
														width="96"
														height="36"
														rx="6"
														fill={isSelected ? "#27272a" : "#18181b"}
														stroke={isSelected
															? "#10b981"
															: node.type === "trait"
																? "#10b981"
																: node.type === "class"
																	? "#3b82f6"
																	: "#a855f7"}
														stroke-width={isSelected ? "2.5" : "1.5"}
														class="transition-all duration-200 group-hover:stroke-white"
													/>
													<text
														x={nx}
														y={ny + 4}
														text-anchor="middle"
														fill={node.type === "trait" ? "#34d399" : node.type === "class" ? "#60a5fa" : "#c084fc"}
														font-size="9"
														font-family="monospace"
														font-weight={isSelected ? "bold" : "normal"}
													>
														{node.name}
													</text>
												{/if}

												<!-- Language Tag Pill above node -->
												<rect x={nx - 18} y={ny - 28} width="36" height="11" rx="3" fill="#27272a" />
												<text x={nx} y={ny - 20} text-anchor="middle" fill="#a1a1aa" font-size="7" font-family="monospace">
													{node.lang}
												</text>
											</g>
										{/if}
									{/each}
								</svg>
							</div>

							<!-- Dynamic Selected Node Inspector Bar -->
							<div class="border-t border-zinc-800 pt-2 flex flex-wrap items-center justify-between gap-2 text-[11px] text-zinc-400">
								<div class="flex items-center gap-2">
									<span class="text-zinc-500 font-mono">Selected:</span>
									<span class="text-white font-bold font-mono">{selectedNode.name}</span>
									<span class="rounded bg-zinc-800 px-1.5 py-0.2 text-[10px] text-zinc-300 font-mono">{selectedNode.lang}</span>
									<span class="text-zinc-500 font-mono hidden sm:inline">({selectedNode.package})</span>
								</div>
								<div class="flex items-center gap-3 font-mono text-[10px]">
									<span>In: <strong class="text-emerald-400">{incomingEdges.length}</strong></span>
									<span>Out: <strong class="text-blue-400">{outgoingEdges.length}</strong></span>
									<span class="text-zinc-500 hidden md:inline">{selectedNode.file}:{selectedNode.line}</span>
								</div>
							</div>
						</div>
					{:else if activeTab === "terminal"}
						<!-- 2. FULLY INTERACTIVE TERMINAL -->
						<div class="space-y-4 text-left">
							<!-- Command Selector Bar -->
							<div class="flex flex-wrap items-center justify-between gap-2 border-b border-zinc-800 pb-3">
								<div class="flex items-center gap-2">
									<span class="text-xs text-zinc-400 font-mono">Run subcommand:</span>
									<button
										type="button"
										onclick={() => (activeCliCommand = "analyze")}
										class={`px-2.5 py-1 rounded text-xs font-mono transition-colors cursor-pointer ${
											activeCliCommand === "analyze"
												? "bg-emerald-500 text-zinc-950 font-bold"
												: "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
										}`}
									>
										omnideps analyze
									</button>
									<button
										type="button"
										onclick={() => (activeCliCommand = "serve")}
										class={`px-2.5 py-1 rounded text-xs font-mono transition-colors cursor-pointer ${
											activeCliCommand === "serve"
												? "bg-emerald-500 text-zinc-950 font-bold"
												: "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
										}`}
									>
										omnideps serve
									</button>
									<button
										type="button"
										onclick={() => (activeCliCommand = "benchmark")}
										class={`px-2.5 py-1 rounded text-xs font-mono transition-colors cursor-pointer ${
											activeCliCommand === "benchmark"
												? "bg-emerald-500 text-zinc-950 font-bold"
												: "bg-zinc-800 text-zinc-300 hover:bg-zinc-700"
										}`}
									>
										omnideps benchmark
									</button>
								</div>
								<span class="text-[11px] text-zinc-500 font-mono">Interactive CLI Sandbox</span>
							</div>

							<!-- Terminal Output Screen -->
							{#if activeCliCommand === "analyze"}
								<div class="space-y-1.5 font-mono text-xs sm:text-sm">
									<div class="flex items-center gap-2 text-zinc-400">
										<span class="text-emerald-400 font-bold">user@workstation</span>
										<span>:~/polyglot-core$</span>
										<span class="text-zinc-100 font-semibold">omnideps analyze ./src -o dependencies.json --csv metrics.csv</span>
									</div>
									<p class="text-zinc-400 text-xs">
										[1/4] <span class="text-zinc-200">Discovered 148 source files across 5 languages:</span>
										<span class="text-emerald-400">(Java: 64, Rust: 42, Python: 22, C++: 12, C: 8)</span>
									</p>
									<p class="text-zinc-400 text-xs">
										[2/4] <span class="text-zinc-200">Parallel Tree-sitter parsing with Rayon (8 threads)...</span>
										<span class="text-emerald-400">Done in 28ms</span>
									</p>
									<p class="text-zinc-400 text-xs">
										[3/4] <span class="text-zinc-200">Extracting AST components & scope trees...</span>
										<span class="text-emerald-400">Done (1,840 entities)</span>
									</p>
									<p class="text-zinc-400 text-xs">
										[4/4] <span class="text-zinc-200">Cross-file & polyglot type resolution...</span>
										<span class="text-emerald-400">Done (5,114 edges resolved)</span>
									</p>
									<div class="my-2 border-t border-zinc-800 pt-2 text-xs">
										<span class="text-emerald-400 font-semibold">✓ Unified output generated:</span>
										<span class="text-zinc-300">dependencies.json, dependencies_cyto.json, metrics.csv</span>
									</div>
								</div>
							{:else if activeCliCommand === "serve"}
								<div class="space-y-1.5 font-mono text-xs sm:text-sm">
									<div class="flex items-center gap-2 text-zinc-400">
										<span class="text-emerald-400 font-bold">user@workstation</span>
										<span>:~/polyglot-core$</span>
										<span class="text-zinc-100 font-semibold">omnideps serve --port 3000</span>
									</div>
									<p class="text-zinc-400 text-xs">
										[+] Initializing embedded Axum HTTP server on 127.0.0.1:3000...
									</p>
									<p class="text-zinc-400 text-xs">
										[+] Unpacking embedded SvelteKit visualizer SPA from binary memory...
									</p>
									<p class="text-emerald-400 font-semibold text-xs">
										🚀 Visualizer listening on http://127.0.0.1:3000
									</p>
									<p class="text-zinc-400 text-xs">
										[+] Launched default operating system browser window. Press Ctrl+C to stop.
									</p>
								</div>
							{:else}
								<div class="space-y-1.5 font-mono text-xs sm:text-sm">
									<div class="flex items-center gap-2 text-zinc-400">
										<span class="text-emerald-400 font-bold">user@workstation</span>
										<span>:~/polyglot-core$</span>
										<span class="text-zinc-100 font-semibold">omnideps benchmark run tests/benchmarks/benchmark-java</span>
									</div>
									<p class="text-zinc-400 text-xs">
										[+] Running ground-truth evaluation on AST abstraction suite...
									</p>
									<p class="text-zinc-300 text-xs">
										Ground truth entities: 320 | Extracted entities: 318
									</p>
									<p class="text-emerald-400 font-semibold text-xs">
										Precision: 99.37% | Recall: 98.75% | F1-Score: 0.9906
									</p>
									<p class="text-zinc-400 text-xs">
										Report saved to: tests/benchmarks/benchmark-java/report.json
									</p>
								</div>
							{/if}
						</div>
					{:else}
						<!-- 3. UNIFIED IR SCHEMA & JSON VIEW -->
						<div class="space-y-3 text-left">
							<div class="flex items-center justify-between border-b border-zinc-800 pb-2">
								<div class="flex items-center gap-2">
									<button
										type="button"
										onclick={() => (schemaMode = "canonical")}
										class={`px-2 py-0.5 rounded text-xs font-mono transition-colors cursor-pointer ${
											schemaMode === "canonical" ? "bg-white text-zinc-950 font-bold" : "bg-zinc-800 text-zinc-300"
										}`}
									>
										Canonical IR
									</button>
									<button
										type="button"
										onclick={() => (schemaMode = "cytoscape")}
										class={`px-2 py-0.5 rounded text-xs font-mono transition-colors cursor-pointer ${
											schemaMode === "cytoscape" ? "bg-white text-zinc-950 font-bold" : "bg-zinc-800 text-zinc-300"
										}`}
									>
										Cytoscape Elements
									</button>
								</div>
								<button
									type="button"
									onclick={() =>
										copyJsonToClipboard(
											schemaMode === "canonical"
												? JSON.stringify({ version: "1.0", language: "polyglot" }, null, 2)
												: "Cytoscape format"
										)}
									class="flex items-center gap-1 text-[11px] text-zinc-400 hover:text-white transition-colors cursor-pointer font-mono"
								>
									{#if copiedJson}
										<CheckIcon class="size-3 text-emerald-400" />
										Copied!
									{:else}
										<CopyIcon class="size-3" />
										Copy JSON
									{/if}
								</button>
							</div>

							<pre class="text-left text-zinc-300 text-xs leading-relaxed font-mono overflow-x-auto max-h-[250px]">
{#if schemaMode === "canonical"}
{`{
  "version": "1.0",
  "project_type": "polyglot",
  "languages_detected": ["Java", "Rust", "Python", "C"],
  "nodes": [
    {
      "id": "java:com.omnideps.db.DatabaseService",
      "language": "Java",
      "kind": "interface",
      "name": "DatabaseService",
      "file": "src/db/DatabaseService.java",
      "line": 12
    },
    {
      "id": "rust:crate::server::AppRouter",
      "language": "Rust",
      "kind": "struct",
      "name": "AppRouter",
      "file": "src/server/router.rs",
      "line": 34
    }
  ],
  "edges": [
    {
      "source": "rust:crate::server::AppRouter",
      "target": "java:com.omnideps.db.DatabaseService",
      "kind": "invokes",
      "inter_language": true
    }
  ]
}`}
{:else}
{`{
  "elements": {
    "nodes": [
      { "data": { "id": "java:DatabaseService", "label": "DatabaseService", "lang": "Java" } },
      { "data": { "id": "rust:AppRouter", "label": "AppRouter", "lang": "Rust" } }
    ],
    "edges": [
      { "data": { "id": "e_01", "source": "rust:AppRouter", "target": "java:DatabaseService", "kind": "invokes" } }
    ]
  }
}`}
{/if}
							</pre>
						</div>
					{/if}
				</div>
			</div>
		</div>
	</section>

	<!-- Dedicated Polyglot Engine Section (Replaced hypothetical comparisons with factual polyglot capabilities) -->
	<section id="polyglot" class="border-b py-20 sm:py-28 bg-muted/20">
		<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
			<div class="mx-auto max-w-3xl text-center">
				<Badge variant="outline" class="mb-3 font-mono text-xs">Unified Polyglot Engine</Badge>
				<h2 class="text-foreground text-3xl font-bold tracking-tight sm:text-4xl">
					Scan multiple languages together into one final graph
				</h2>
				<p class="text-muted-foreground mt-4 text-base sm:text-lg">
					Modern applications are rarely built with a single language. Omnideps parses source files across your entire codebase simultaneously and links them in a single, comprehensive architectural model.
				</p>
			</div>

			<div class="mt-14 grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
				<!-- Card 1 -->
				<div class="border-border bg-card rounded-xl border p-6 shadow-xs">
					<div class="bg-foreground text-background mb-4 flex size-9 items-center justify-center rounded-lg font-mono text-xs font-bold">
						01
					</div>
					<h3 class="text-lg font-bold text-foreground">Cross-Language Unification</h3>
					<p class="text-muted-foreground mt-2 text-sm leading-relaxed">
						Whether your project combines a <strong>Java</strong> backend with <strong>Rust</strong> modules, <strong>Python</strong> microservices, or native <strong>C/C++</strong> libraries, Omnideps scans all files in a single pass into one unified graph.
					</p>
				</div>

				<!-- Card 2 -->
				<div class="border-border bg-card rounded-xl border p-6 shadow-xs">
					<div class="bg-foreground text-background mb-4 flex size-9 items-center justify-center rounded-lg font-mono text-xs font-bold">
						02
					</div>
					<h3 class="text-lg font-bold text-foreground">Direct AST Syntax Extraction</h3>
					<p class="text-muted-foreground mt-2 text-sm leading-relaxed">
						Unlike tools that merely read manifest files, Omnideps parses the concrete syntax tree of every source file using Tree-sitter, extracting classes, traits, functions, methods, and call sites.
					</p>
				</div>

				<!-- Card 3 -->
				<div class="border-border bg-card rounded-xl border p-6 shadow-xs">
					<div class="bg-foreground text-background mb-4 flex size-9 items-center justify-center rounded-lg font-mono text-xs font-bold">
						03
					</div>
					<h3 class="text-lg font-bold text-foreground">Parallel Thread Engine</h3>
					<p class="text-muted-foreground mt-2 text-sm leading-relaxed">
						Powered by Rayon and written in Rust, file discovery and grammar dispatch execute across all CPU threads simultaneously, parsing hundreds of files in milliseconds.
					</p>
				</div>

				<!-- Card 4 -->
				<div class="border-border bg-card rounded-xl border p-6 shadow-xs">
					<div class="bg-foreground text-background mb-4 flex size-9 items-center justify-center rounded-lg font-mono text-xs font-bold">
						04
					</div>
					<h3 class="text-lg font-bold text-foreground">Canonical Intermediate IR</h3>
					<p class="text-muted-foreground mt-2 text-sm leading-relaxed">
						Every extracted language entity is normalized into a unified schema of containers (packages, modules, classes), callables, and typed dependencies for uniform metric computation.
					</p>
				</div>

				<!-- Card 5 -->
				<div class="border-border bg-card rounded-xl border p-6 shadow-xs">
					<div class="bg-foreground text-background mb-4 flex size-9 items-center justify-center rounded-lg font-mono text-xs font-bold">
						05
					</div>
					<h3 class="text-lg font-bold text-foreground">Type & Scope Resolution</h3>
					<p class="text-muted-foreground mt-2 text-sm leading-relaxed">
						Resolves symbol references across file boundaries, tracking inheritance contracts, interface implementations, and generic type parameters.
					</p>
				</div>

				<!-- Card 6 -->
				<div class="border-border bg-card rounded-xl border p-6 shadow-xs">
					<div class="bg-foreground text-background mb-4 flex size-9 items-center justify-center rounded-lg font-mono text-xs font-bold">
						06
					</div>
					<h3 class="text-lg font-bold text-foreground">Built-in Web Explorer</h3>
					<p class="text-muted-foreground mt-2 text-sm leading-relaxed">
						Instantly launch <code>omnideps serve</code> to view the polyglot dependency graph with force-directed physics, interactive filtering, and cycle detection.
					</p>
				</div>
			</div>
		</div>
	</section>

	<!-- Features Bento Grid -->
	<section id="features" class="border-b py-20 sm:py-28">
		<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
			<div class="mx-auto max-w-3xl text-center">
				<Badge variant="outline" class="mb-3 font-mono text-xs">Core Highlights</Badge>
				<h2 class="text-foreground text-3xl font-bold tracking-tight sm:text-4xl">
					Engineered for speed, precision, and simplicity
				</h2>
				<p class="text-muted-foreground mt-4 text-base sm:text-lg">
					Built in Rust to provide instantaneous dependency analysis for both single repositories and polyglot monorepos.
				</p>
			</div>

			<div class="mt-14 grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
				{#each features as f}
					<div class="border-border bg-card group rounded-xl border p-6 transition-all hover:border-foreground/30 hover:shadow-md">
						<div class="bg-muted text-foreground mb-4 flex size-11 items-center justify-center rounded-lg border">
							<f.icon class="size-5" />
						</div>
						<h3 class="text-foreground font-semibold text-base tracking-tight">{f.title}</h3>
						<p class="text-muted-foreground mt-2 text-sm leading-relaxed">{f.description}</p>
					</div>
				{/each}
			</div>
		</div>
	</section>

	<!-- Supported Languages Matrix: Java first (Stable), all others Experimental -->
	<section class="border-b py-20 sm:py-28 bg-muted/20">
		<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
			<div class="mx-auto max-w-3xl text-center">
				<Badge variant="outline" class="mb-3 font-mono text-xs">Language Ecosystem</Badge>
				<h2 class="text-foreground text-3xl font-bold tracking-tight sm:text-4xl">
					Multi-Language Native Support
				</h2>
				<p class="text-muted-foreground mt-4 text-base sm:text-lg">
					Java is fully stable and production-ready. Additional languages are currently in active experimental development.
				</p>
			</div>

			<div class="mt-12 overflow-x-auto">
				<div class="border-border bg-card inline-block min-w-full align-middle rounded-xl border shadow-xs overflow-hidden">
					<table class="min-w-full divide-y divide-border text-left text-sm">
						<thead class="bg-muted/60 text-muted-foreground text-xs font-semibold uppercase tracking-wider">
							<tr>
								<th scope="col" class="px-6 py-4">Language</th>
								<th scope="col" class="px-6 py-4">Tree-sitter Parser</th>
								<th scope="col" class="px-6 py-4">Extracted Entities</th>
								<th scope="col" class="px-6 py-4">Resolution Capabilities</th>
								<th scope="col" class="px-6 py-4 text-right">Status</th>
							</tr>
						</thead>
						<tbody class="divide-y divide-border">
							{#each languages as lang}
								<tr class="hover:bg-muted/30 transition-colors">
									<td class="whitespace-nowrap px-6 py-4 font-bold text-foreground flex items-center gap-2">
										<span>{lang.name}</span>
										{#if lang.status === "Stable"}
											<span class="size-2 rounded-full bg-emerald-500" title="Stable Production Support"></span>
										{/if}
									</td>
									<td class="whitespace-nowrap px-6 py-4 font-mono text-xs text-muted-foreground">{lang.grammar}</td>
									<td class="px-6 py-4 text-xs text-muted-foreground">{lang.entities}</td>
									<td class="px-6 py-4 text-xs text-foreground">{lang.features}</td>
									<td class="whitespace-nowrap px-6 py-4 text-right">
										{#if lang.status === "Stable"}
											<Badge class="bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30 font-mono text-[10px]">
												Stable
											</Badge>
										{:else}
											<Badge variant="outline" class="border-amber-500/40 text-amber-600 dark:text-amber-400 font-mono text-[10px]">
												Experimental
											</Badge>
										{/if}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</div>
		</div>
	</section>

	<!-- Quick 3-Step Workflow -->
	<section id="quickstart" class="border-b py-20 sm:py-28">
		<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
			<div class="mx-auto max-w-3xl text-center">
				<Badge variant="outline" class="mb-3 font-mono text-xs">Fast Path</Badge>
				<h2 class="text-foreground text-3xl font-bold tracking-tight sm:text-4xl">
					Up and running in 3 steps
				</h2>
				<p class="text-muted-foreground mt-4 text-base sm:text-lg">
					Zero configuration needed to get your first architectural graph.
				</p>
			</div>

			<div class="mt-14 grid gap-8 md:grid-cols-3">
				<!-- Step 1 -->
				<div class="border-border bg-card rounded-xl border p-6 flex flex-col justify-between">
					<div>
						<span class="font-mono text-3xl font-extrabold text-muted-foreground/40">01</span>
						<h3 class="text-foreground mt-4 text-lg font-bold">Download Binary</h3>
						<p class="text-muted-foreground mt-2 text-sm leading-relaxed">
							Grab the pre-compiled executable for macOS, Linux, or Windows from GitHub Releases. No runtime or build dependencies required.
						</p>
					</div>
					<div class="mt-6 border-t pt-4">
						<a href="/docs/getting-started/installation" class="text-xs font-semibold text-foreground hover:underline inline-flex items-center gap-1">
							Installation guide <ArrowRightIcon class="size-3" />
						</a>
					</div>
				</div>

				<!-- Step 2 -->
				<div class="border-border bg-card rounded-xl border p-6 flex flex-col justify-between">
					<div>
						<span class="font-mono text-3xl font-extrabold text-muted-foreground/40">02</span>
						<h3 class="text-foreground mt-4 text-lg font-bold">Analyze Codebase</h3>
						<p class="text-muted-foreground mt-2 text-sm leading-relaxed">
							Run <code>omnideps analyze ./project</code>. The multi-threaded engine parses ASTs and generates JSON + Cytoscape graph models.
						</p>
					</div>
					<div class="mt-6 border-t pt-4">
						<a href="/docs/cli/analyze" class="text-xs font-semibold text-foreground hover:underline inline-flex items-center gap-1">
							CLI options & flags <ArrowRightIcon class="size-3" />
						</a>
					</div>
				</div>

				<!-- Step 3 -->
				<div class="border-border bg-card rounded-xl border p-6 flex flex-col justify-between">
					<div>
						<span class="font-mono text-3xl font-extrabold text-muted-foreground/40">03</span>
						<h3 class="text-foreground mt-4 text-lg font-bold">Explore & Visualize</h3>
						<p class="text-muted-foreground mt-2 text-sm leading-relaxed">
							Execute <code>omnideps serve</code>. Omnideps launches its embedded web server and immediately opens your interactive Cytoscape graph.
						</p>
					</div>
					<div class="mt-6 border-t pt-4">
						<a href="/docs/cli/serve" class="text-xs font-semibold text-foreground hover:underline inline-flex items-center gap-1">
							Visualizer guide <ArrowRightIcon class="size-3" />
						</a>
					</div>
				</div>
			</div>
		</div>
	</section>

	<!-- Call to Action Banner -->
	<section class="py-20 sm:py-28">
		<div class="mx-auto max-w-5xl px-4 text-center sm:px-6 lg:px-8">
			<h2 class="text-foreground text-3xl font-extrabold tracking-tight sm:text-5xl">
				Ready to inspect your software architecture?
			</h2>
			<p class="text-muted-foreground mx-auto mt-4 max-w-2xl text-base sm:text-lg">
				Download the latest standalone binary or clone the repository to contribute.
			</p>
			<div class="mt-8 flex flex-wrap items-center justify-center gap-4">
				<Button size="lg" href="/docs/getting-started/installation" class="gap-2 px-6">
					<DownloadIcon class="size-4" />
					Get Omnideps
				</Button>
				<Button
					variant="outline"
					size="lg"
					href="https://github.com/FerrarioChristian/omnideps"
					target="_blank"
					class="gap-2 px-6"
				>
					<GithubIcon class="size-4" />
					Star on GitHub
				</Button>
			</div>
		</div>
	</section>
</main>

<!-- Footer -->
<footer class="border-border border-t bg-muted/30 py-12 text-sm">
	<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
		<div class="flex flex-col items-center justify-between gap-6 md:flex-row">
			<div class="flex items-center gap-3">
				<div class="bg-foreground text-background flex size-6 items-center justify-center rounded font-mono text-xs font-bold">
					⬡
				</div>
				<span class="font-semibold text-foreground">Omnideps</span>
				<span class="text-muted-foreground text-xs">• MIT License</span>
			</div>

			<div class="flex flex-wrap items-center gap-6 text-xs text-muted-foreground">
				<a href="/docs" class="hover:text-foreground transition-colors">Documentation</a>
				<a href="/docs/cli/analyze" class="hover:text-foreground transition-colors">CLI Reference</a>
				<a href="/docs/architecture/overview" class="hover:text-foreground transition-colors">Architecture</a>
				<a
					href="https://github.com/FerrarioChristian/omnideps/releases"
					target="_blank"
					class="hover:text-foreground transition-colors"
				>
					Releases
				</a>
				<a
					href="https://github.com/FerrarioChristian/omnideps"
					target="_blank"
					class="hover:text-foreground transition-colors"
				>
					GitHub
				</a>
			</div>
		</div>

		<div class="mt-8 border-t border-border pt-6 text-center text-xs text-muted-foreground">
			Developed by Christian Ferrario as part of the Master's Thesis research at Università degli Studi di Milano-Bicocca.
		</div>
	</div>
</footer>
