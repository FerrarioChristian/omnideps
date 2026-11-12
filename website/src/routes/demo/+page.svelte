<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Badge } from '$lib/components/ui/badge/index.js';
	import DarkModeSwitcher from '$lib/components/theme/dark-mode-switcher.svelte';
	import GraphViewer from '$lib/components/demo/GraphViewer.svelte';
	import { toast } from 'svelte-sonner';

	// Lucide Icons
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import PlayIcon from '@lucide/svelte/icons/play';
	import RotateCcwIcon from '@lucide/svelte/icons/rotate-ccw';
	import GithubIcon from '@lucide/svelte/icons/github';
	import CodeIcon from '@lucide/svelte/icons/code';
	import ServerIcon from '@lucide/svelte/icons/server';
	import SparklesIcon from '@lucide/svelte/icons/sparkles';

	interface Preset {
		label: string;
		lang: string;
		code: string;
	}

	const PRESETS: Record<string, Preset> = {
		java_default: {
			label: 'Java: Interfaces & Nested Classes (Default)',
			lang: 'java',
			code: `package demo;

// 1. Relazioni di interfaccia e contratti
interface PaymentMethod {
    boolean process(double amount);
}

// 2. Ereditarietà e classi astratte (IsA / Implements)
abstract class BaseAccount implements PaymentMethod {
    protected String accountId;
    
    public BaseAccount(String id) {
        this.accountId = id;
    }
}

// 3. Classe concreta con classe annidata (NestedIn)
class CreditCard extends BaseAccount {
    private SecurityToken token;

    public CreditCard(String id, String rawSecret) {
        super(id);
        this.token = new SecurityToken(rawSecret);
    }

    @Override
    public boolean process(double amount) {
        return token.validate();
    }

    // Inner class statica
    static class SecurityToken {
        private String secret;
        public SecurityToken(String secret) { this.secret = secret; }
        public boolean validate() { return secret != null; }
    }
}

// 4. Servizio che usa e istanzia i componenti (Instantiates / Calls)
class CheckoutService {
    public void executeCheckout(PaymentMethod method, double total) {
        CreditCard fallback = new CreditCard("CC-99", "secret-key");
        method.process(total);
        fallback.process(total);
    }
}`
		},
		rust_traits: {
			label: 'Rust: Traits, Structs & Generics',
			lang: 'rs',
			code: `pub trait Storage {
    fn save(&self, data: &str);
}

pub struct Database {
    pub connection_url: String,
}

impl Storage for Database {
    fn save(&self, data: &str) {
        println!("Saving to DB: {}", data);
    }
}

pub struct Service<S: Storage> {
    storage: S,
}

impl<S: Storage> Service<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub fn handle(&self, text: &str) {
        self.storage.save(text);
    }
}`
		},
		python_oop: {
			label: 'Python: ABC & Dependency Injection',
			lang: 'py',
			code: `from abc import ABC, abstractmethod

class BaseRepository(ABC):
    @abstractmethod
    def fetch(self, id: str):
        pass

class SqlRepository(BaseRepository):
    def __init__(self, dsn: str):
        self.dsn = dsn

    def fetch(self, id: str):
        return f"Entity {id} from {self.dsn}"

class UserService:
    def __init__(self, repo: BaseRepository):
        self.repo = repo

    def get_user(self, user_id: str):
        return self.repo.fetch(user_id)
`
		}
	};

	let selectedPreset = $state('java_default');
	let selectedLang = $state('java');
	let rawCode = $state(PRESETS.java_default.code);

	let elements = $state<any[]>([]);
	let rawOutput = $state<any>(null);
	let statusMessage = $state('Loading precomputed demo graph...');
	let isAnalyzing = $state(false);
	let isBackendActive = $state<boolean | null>(null);

	onMount(async () => {
		// 1. Immediately load precomputed default JSON for instantaneous experience
		try {
			const res = await fetch('/demo-default.json');
			if (res.ok) {
				const data = await res.json();
				elements = Array.isArray(data) ? data : data.elements || [];
				statusMessage = 'Sample: Java Interfaces & Hierarchies (Precomputed)';
			}
		} catch (err) {
			console.warn('Could not load static demo graph', err);
			statusMessage = 'Ready. Paste code and click Analyze.';
		}
	});

	function handlePresetChange(presetKey: string) {
		selectedPreset = presetKey;
		const preset = PRESETS[presetKey];
		if (preset) {
			selectedLang = preset.lang;
			rawCode = preset.code;
			if (presetKey === 'java_default') {
				// Re-load instant precomputed graph
				fetch('/demo-default.json')
					.then((r) => r.json())
					.then((data) => {
						elements = Array.isArray(data) ? data : data.elements || [];
						statusMessage = 'Sample: Java Interfaces & Hierarchies (Precomputed)';
					})
					.catch(() => {});
			}
		}
	}

	async function analyzeCode() {
		if (!rawCode.trim()) {
			toast.error('Code snippet cannot be empty');
			return;
		}

		isAnalyzing = true;
		statusMessage = `Analyzing ${selectedLang.toUpperCase()} snippet with Omnideps engine...`;

		try {
			const res = await fetch('/api/analyze', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					code: rawCode,
					extension: selectedLang
				})
			});

			const data = await res.json();

			if (!res.ok) {
				throw new Error(data.error || `HTTP error ${res.status}`);
			}

			elements = data.elements || [];
			rawOutput = data.rawOutput || null;
			isBackendActive = true;
			statusMessage = `Analyzed successfully (${elements.length} graph elements)`;
			toast.success('Dependency graph generated successfully!');
		} catch (err: any) {
			isBackendActive = false;
			statusMessage = `Error: ${err.message}`;
			toast.error(`Analysis failed: ${err.message}`);
		} finally {
			isAnalyzing = false;
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		// Support Tab key indentation inside textarea
		if (e.key === 'Tab') {
			e.preventDefault();
			const target = e.target as HTMLTextAreaElement;
			const start = target.selectionStart;
			const end = target.selectionEnd;
			rawCode = rawCode.substring(0, start) + '    ' + rawCode.substring(end);
			requestAnimationFrame(() => {
				target.selectionStart = target.selectionEnd = start + 4;
			});
		}
	}
</script>

<svelte:head>
	<title>Omnideps — Interactive Online Demo & Code Playground</title>
	<meta
		name="description"
		content="Test Omnideps live in your browser: analyze code snippets and explore AST dependency graphs in real time."
	/>
</svelte:head>

<!-- Top Demo Navigation Bar -->
<header class="bg-background/90 border-border sticky top-0 z-50 flex h-14 w-full items-center justify-between border-b px-4 backdrop-blur-md">
	<div class="flex items-center gap-3">
		<Button variant="ghost" size="sm" href="/" class="gap-1.5 text-xs">
			<ArrowLeftIcon class="size-3.5" />
			Back to Website
		</Button>
		<div class="bg-border h-4 w-px"></div>
		<div class="flex items-center gap-2">
			<div class="bg-foreground text-background flex size-6 items-center justify-center rounded-md font-mono text-xs font-bold">
				⬡
			</div>
			<span class="text-sm font-semibold tracking-tight">Omnideps Playground</span>
			<Badge variant="secondary" class="font-mono text-[10px] hidden sm:inline-flex">Live Demo</Badge>
		</div>
	</div>

	<div class="flex items-center gap-2">
		{#if isBackendActive === true}
			<Badge variant="outline" class="border-emerald-500/50 text-emerald-500 gap-1 text-[11px] hidden md:inline-flex">
				<span class="size-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
				Backend Connected
			</Badge>
		{:else if isBackendActive === false}
			<Badge variant="outline" class="border-amber-500/50 text-amber-500 gap-1 text-[11px] hidden md:inline-flex">
				<span class="size-1.5 rounded-full bg-amber-500"></span>
				Backend Offline
			</Badge>
		{/if}

		<Button variant="ghost" size="sm" href="/docs" class="text-xs">Documentation</Button>
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
	</div>
</header>

<!-- Main Container for Visualizer -->
<main class="relative h-[calc(100vh-3.5rem)] w-full overflow-hidden bg-[#1e1e1e]">
	{#snippet controls()}
		<div class="space-y-3.5 mb-4">
			<!-- Preset Selector -->
			<div class="space-y-1">
				<label for="preset-select" class="text-xs font-medium text-gray-300 block">Preset Examples:</label>
				<select
					id="preset-select"
					bind:value={selectedPreset}
					onchange={(e) => handlePresetChange((e.target as HTMLSelectElement).value)}
					class="w-full text-xs"
				>
					{#each Object.entries(PRESETS) as [key, item]}
						<option value={key}>{item.label}</option>
					{/each}
				</select>
			</div>

			<!-- Language Selector -->
			<div class="space-y-1">
				<label for="lang-select" class="text-xs font-medium text-gray-300 block">Language:</label>
				<select id="lang-select" bind:value={selectedLang} class="w-full text-xs">
					<option value="java">Java (.java)</option>
					<option value="rs">Rust (.rs)</option>
					<option value="py">Python (.py)</option>
					<option value="c">C (.c)</option>
					<option value="cpp">C++ (.cpp)</option>
				</select>
			</div>

			<!-- Code Editor Textarea -->
			<div class="space-y-1">
				<div class="flex items-center justify-between text-xs text-gray-400">
					<span>Source Snippet:</span>
					<span class="font-mono text-[10px]">{rawCode.length} chars</span>
				</div>
				<textarea
					bind:value={rawCode}
					onkeydown={handleKeydown}
					placeholder="Paste your source code here..."
					rows="9"
					spellcheck="false"
					style="width: 100%; font-family: monospace; font-size: 11px; line-height: 1.4; background: #18181b; color: #f4f4f5; border: 1px solid #3f3f46; border-radius: 6px; padding: 8px; resize: vertical; box-sizing: border-box;"
				></textarea>
			</div>

			<!-- Actions -->
			<div class="flex gap-2 pt-1">
				<button
					onclick={analyzeCode}
					disabled={isAnalyzing}
					class="analyze-btn"
					style="flex: 1; display: flex; align-items: center; justify-content: center; gap: 6px;"
				>
					{#if isAnalyzing}
						<span class="size-3 border-2 border-white/20 border-t-white rounded-full animate-spin"></span>
						Analyzing...
					{:else}
						<PlayIcon class="size-3.5 fill-current" />
						Analyze Code
					{/if}
				</button>
				<button
					onclick={() => handlePresetChange(selectedPreset)}
					title="Reset to preset default"
					class="export-btn"
					style="width: auto; padding: 6px 10px;"
				>
					<RotateCcwIcon class="size-3.5" />
				</button>
			</div>
		</div>
	{/snippet}

	<!-- Render the Cytoscape Graph Viewer -->
	<GraphViewer {elements} {rawOutput} {statusMessage} {controls} />
</main>
