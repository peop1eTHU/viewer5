<script lang="ts">
  import { onMount } from "svelte";
  import { convertFileSrc, invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Button, RatingGroup, ScrollArea, Select, Tabs } from "bits-ui";

  type ThemeMode =
    | "light"
    | "dark"
    | "mono"
    | "vivid"
    | "ocean"
    | "sunset"
    | "aurora"
    | "carnival";

  type MediaMode = "images" | "videos" | "mixed";
  type ViewMode = "viewer" | "waterfall" | "mosaic";

  type IncludeFolder = {
    path: string;
    stars: number;
  };

  type PresetRecord = {
    name: string;
    includeFolders: IncludeFolder[];
    excludeFolders: string[];
    mediaMode: MediaMode;
  };

  type MediaResponse = {
    path: string;
    fileName: string;
    mediaType: "image" | "video";
    preloadPaths: string[];
    index: number;
    total: number;
    mode: "global" | "folder";
    focusedFolder: string;
  };

  type MediaCatalogItem = {
    path: string;
    fileName: string;
    mediaType: "image" | "video";
  };

  type LibrarySummary = {
    total: number;
    includeFolders: IncludeFolder[];
    excludeFolders: string[];
    mode: "global" | "folder";
    focusedFolder: string;
    mediaMode: MediaMode;
  };

  const STAR_MAX = 5;

  const themeClass: Record<ThemeMode, string> = {
    light: "bg-slate-100 text-slate-900",
    dark: "bg-slate-950 text-slate-100",
    mono: "bg-teal-50 text-teal-950",
    vivid: "bg-rose-50 text-fuchsia-950",
    ocean: "bg-cyan-950 text-cyan-50",
    sunset: "bg-amber-50 text-orange-950",
    aurora: "bg-gradient-to-br from-sky-50 via-emerald-50 to-teal-100 text-slate-900",
    carnival: "bg-gradient-to-br from-pink-50 via-violet-50 to-sky-50 text-slate-900",
  };

  const panelClass: Record<ThemeMode, string> = {
    light: "bg-white border-slate-200",
    dark: "bg-slate-900 border-slate-700",
    mono: "bg-white border-teal-200",
    vivid: "bg-white border-fuchsia-200",
    ocean: "bg-cyan-900 border-cyan-700",
    sunset: "bg-white border-orange-200",
    aurora: "bg-white/90 border-emerald-200",
    carnival: "bg-white/90 border-violet-200",
  };

  const softClass: Record<ThemeMode, string> = {
    light: "bg-slate-50 text-slate-700",
    dark: "bg-slate-800 text-slate-200",
    mono: "bg-teal-100 text-teal-800",
    vivid: "bg-fuchsia-100 text-fuchsia-800",
    ocean: "bg-cyan-800 text-cyan-100",
    sunset: "bg-orange-100 text-orange-800",
    aurora: "bg-emerald-100/80 text-emerald-900",
    carnival: "bg-violet-100/80 text-violet-900",
  };

  const controlClass: Record<ThemeMode, string> = {
    light: "bg-white text-slate-900 border-slate-200",
    dark: "bg-slate-900 text-slate-100 border-slate-700",
    mono: "bg-white text-teal-900 border-teal-200",
    vivid: "bg-white text-fuchsia-900 border-fuchsia-200",
    ocean: "bg-cyan-900 text-cyan-50 border-cyan-700",
    sunset: "bg-white text-orange-900 border-orange-200",
    aurora: "bg-white text-emerald-900 border-emerald-200",
    carnival: "bg-white text-violet-900 border-violet-200",
  };

  const accentClass: Record<ThemeMode, string> = {
    light: "bg-slate-900 text-white border-slate-900",
    dark: "bg-slate-100 text-slate-950 border-slate-100",
    mono: "bg-teal-700 text-teal-50 border-teal-700",
    vivid: "bg-rose-500 text-white border-rose-500",
    ocean: "bg-cyan-300 text-cyan-950 border-cyan-300",
    sunset: "bg-orange-500 text-white border-orange-500",
    aurora: "bg-emerald-500 text-white border-emerald-500",
    carnival: "bg-violet-500 text-white border-violet-500",
  };

  const menuContentClass: Record<ThemeMode, string> = {
    light: "z-50 w-[250px] rounded-xl border border-slate-200 bg-white p-1 text-slate-900 shadow-xl",
    dark: "z-50 w-[250px] rounded-xl border border-slate-700 bg-slate-900 p-1 text-slate-100 shadow-2xl",
    mono: "z-50 w-[250px] rounded-xl border border-teal-200 bg-white p-1 text-teal-950 shadow-xl",
    vivid: "z-50 w-[250px] rounded-xl border border-fuchsia-200 bg-white p-1 text-fuchsia-950 shadow-xl",
    ocean: "z-50 w-[250px] rounded-xl border border-cyan-700 bg-cyan-900 p-1 text-cyan-100 shadow-2xl",
    sunset: "z-50 w-[250px] rounded-xl border border-orange-200 bg-white p-1 text-orange-900 shadow-xl",
    aurora: "z-50 w-[250px] rounded-xl border border-emerald-200 bg-white p-1 text-emerald-900 shadow-xl",
    carnival: "z-50 w-[250px] rounded-xl border border-violet-200 bg-white p-1 text-violet-900 shadow-xl",
  };

  const menuItemClass: Record<ThemeMode, string> = {
    light: "cursor-pointer rounded-lg px-3 py-2 text-sm text-slate-900 hover:bg-slate-100",
    dark: "cursor-pointer rounded-lg px-3 py-2 text-sm text-slate-100 hover:bg-slate-800",
    mono: "cursor-pointer rounded-lg px-3 py-2 text-sm text-teal-900 hover:bg-teal-100",
    vivid: "cursor-pointer rounded-lg px-3 py-2 text-sm text-fuchsia-900 hover:bg-fuchsia-100",
    ocean: "cursor-pointer rounded-lg px-3 py-2 text-sm text-cyan-100 hover:bg-cyan-800",
    sunset: "cursor-pointer rounded-lg px-3 py-2 text-sm text-orange-900 hover:bg-orange-100",
    aurora: "cursor-pointer rounded-lg px-3 py-2 text-sm text-emerald-900 hover:bg-emerald-100",
    carnival: "cursor-pointer rounded-lg px-3 py-2 text-sm text-violet-900 hover:bg-violet-100",
  };

  const includeCardClass: Record<ThemeMode, string> = {
    light: "bg-slate-100/90",
    dark: "bg-slate-800/90",
    mono: "bg-teal-100/90",
    vivid: "bg-fuchsia-100/90",
    ocean: "bg-cyan-800/90",
    sunset: "bg-orange-100/90",
    aurora: "bg-emerald-100/90",
    carnival: "bg-violet-100/90",
  };

  const starActiveClass: Record<ThemeMode, string> = {
    light: "text-slate-900",
    dark: "text-slate-100",
    mono: "text-teal-700",
    vivid: "text-rose-500",
    ocean: "text-cyan-300",
    sunset: "text-orange-500",
    aurora: "text-emerald-500",
    carnival: "text-violet-500",
  };

  const starInactiveClass: Record<ThemeMode, string> = {
    light: "text-slate-300",
    dark: "text-slate-600",
    mono: "text-teal-300",
    vivid: "text-fuchsia-300",
    ocean: "text-cyan-700",
    sunset: "text-orange-300",
    aurora: "text-emerald-300",
    carnival: "text-violet-300",
  };

  let includeFolders = $state<IncludeFolder[]>([]);
  let excludeFolders = $state<string[]>([]);
  let mediaMode = $state<MediaMode>("mixed");
  let themeMode = $state<ThemeMode>("light");
  let viewMode = $state<ViewMode>("viewer");

  let currentMedia = $state<MediaResponse | null>(null);
  let currentMediaSrc = $state("");
  let catalog = $state<MediaCatalogItem[]>([]);
  let summary = $state<LibrarySummary>({
    total: 0,
    includeFolders: [],
    excludeFolders: [],
    mode: "global",
    focusedFolder: "",
    mediaMode: "mixed",
  });

  let presets = $state<PresetRecord[]>([]);
  let selectedPreset = $state("");
  let presetDraft = $state("");
  let isBusy = $state(false);
  let errorText = $state("");
  let autoLoadSig = "";

  const modeLabel = $derived(summary.mode === "folder" ? "Folder Flow" : "Global Flow");
  const indexLabel = $derived(currentMedia ? `${currentMedia.index + 1} / ${currentMedia.total}` : "0 / 0");
  const imageCatalog = $derived(catalog.filter((item) => item.mediaType === "image"));
  const mediaModeLabel = $derived(
    mediaMode === "images" ? "Images only" : mediaMode === "videos" ? "Videos only" : "Mixed",
  );

  const warmedGalleryPaths = new Set<string>();

  onMount(() => {
    void refreshPresets();
  });

  $effect(() => {
    if (summary.total <= 0) return;
    if (viewMode === "waterfall" || viewMode === "mosaic") {
      void warmGalleryCache(260);
    }
  });

  $effect(() => {
    const sig = JSON.stringify({ includeFolders, excludeFolders });
    if (sig === autoLoadSig) return;
    autoLoadSig = sig;
    if (includeFolders.length === 0) return;

    const timer = setTimeout(() => {
      void loadLibrary();
    }, 220);

    return () => clearTimeout(timer);
  });

  function normalizePath(path: string): string {
    return path.replace(/\\/g, "/");
  }

  function setCurrentMedia(response: MediaResponse | null) {
    currentMedia = response;
    currentMediaSrc = response ? convertFileSrc(response.path) : "";

    if (!response) return;
    for (const preloadPath of response.preloadPaths) {
      if (response.mediaType === "image") {
        const image = new Image();
        image.src = convertFileSrc(preloadPath);
      }
    }
  }

  async function refreshLibraryMeta() {
    summary = await invoke<LibrarySummary>("library_summary");
    catalog = await invoke<MediaCatalogItem[]>("media_catalog");
  }

  async function refreshPresets() {
    presets = await invoke<PresetRecord[]>("list_presets");
    if (presets.length > 0 && !presets.some((preset) => preset.name === selectedPreset)) {
      selectedPreset = presets[0].name;
      if (!presetDraft.trim()) {
        presetDraft = presets[0].name;
      }
    }
    if (presets.length === 0) {
      selectedPreset = "";
      if (!presetDraft.trim()) {
        presetDraft = "";
      }
    }
  }

  async function runPresetAction(task: () => Promise<void>) {
    isBusy = true;
    errorText = "";
    try {
      await task();
      await refreshPresets();
    } catch (error) {
      errorText = String(error);
    } finally {
      isBusy = false;
    }
  }

  async function savePreset() {
    const name = presetDraft.trim();
    if (!name) {
      errorText = "Enter a preset name before saving.";
      return;
    }

    await runPresetAction(async () => {
      presets = await invoke<PresetRecord[]>("save_preset", {
        name,
        includeFolders,
        excludeFolders,
        mediaMode,
      });
      selectedPreset = name;
      presetDraft = name;
    });
  }

  async function renamePreset() {
    if (!selectedPreset) return;
    const newName = presetDraft.trim();
    if (!newName || newName === selectedPreset) return;

    await runPresetAction(async () => {
      presets = await invoke<PresetRecord[]>("rename_preset", {
        oldName: selectedPreset,
        newName,
      });
      selectedPreset = newName;
      presetDraft = newName;
    });
  }

  async function deletePreset() {
    if (!selectedPreset) return;

    await runPresetAction(async () => {
      presets = await invoke<PresetRecord[]>("delete_preset", { name: selectedPreset });
      selectedPreset = presets[0]?.name ?? "";
    });
  }

  async function loadPresetByName(name: string) {
    if (!name) return;

    await runPresetAction(async () => {
      const preset = await invoke<PresetRecord>("load_preset", { name });
      includeFolders = preset.includeFolders.map((item) => ({ path: item.path, stars: item.stars }));
      excludeFolders = [...preset.excludeFolders];
      mediaMode = preset.mediaMode;
      selectedPreset = preset.name;
      presetDraft = preset.name;
    });
  }

  async function pickIncludeFolder() {
    const selected = await open({ directory: true, multiple: false, title: "Add include folder" });
    if (!selected || typeof selected !== "string") return;

    const path = normalizePath(selected);
    if (includeFolders.some((entry) => entry.path === path)) return;
    includeFolders = [...includeFolders, { path, stars: 3 }];
  }

  async function pickExcludeFolder() {
    const selected = await open({ directory: true, multiple: false, title: "Add excluded folder" });
    if (!selected || typeof selected !== "string") return;

    const path = normalizePath(selected);
    if (excludeFolders.includes(path)) return;
    excludeFolders = [...excludeFolders, path];
  }

  function removeIncludeFolder(path: string) {
    includeFolders = includeFolders.filter((entry) => entry.path !== path);
  }

  function removeExcludeFolder(path: string) {
    excludeFolders = excludeFolders.filter((entry) => entry !== path);
  }

  function setFolderStars(path: string, stars: number) {
    includeFolders = includeFolders.map((entry) =>
      entry.path === path ? { ...entry, stars } : entry,
    );
  }

  async function loadLibrary() {
    if (includeFolders.length === 0) return;

    isBusy = true;
    errorText = "";
    warmedGalleryPaths.clear();
    try {
      const response = await invoke<MediaResponse>("build_library", {
        includeFolders,
        excludeFolders,
        mediaMode,
      });
      setCurrentMedia(response);
      await refreshLibraryMeta();
      void warmGalleryCache(300);
    } catch (error) {
      setCurrentMedia(null);
      errorText = String(error);
    } finally {
      isBusy = false;
    }
  }

  async function nextMedia() {
    if (isBusy) return;
    isBusy = true;
    errorText = "";
    try {
      const response = await invoke<MediaResponse>("next_image");
      setCurrentMedia(response);
      summary = await invoke<LibrarySummary>("library_summary");
    } catch (error) {
      errorText = String(error);
    } finally {
      isBusy = false;
    }
  }

  async function previousMedia() {
    if (isBusy) return;
    isBusy = true;
    errorText = "";
    try {
      const response = await invoke<MediaResponse>("previous_image");
      setCurrentMedia(response);
      summary = await invoke<LibrarySummary>("library_summary");
    } catch (error) {
      errorText = String(error);
    } finally {
      isBusy = false;
    }
  }

  async function reshuffle() {
    if (isBusy) return;
    isBusy = true;
    errorText = "";
    try {
      const response = await invoke<MediaResponse>("reshuffle_library");
      setCurrentMedia(response);
      summary = await invoke<LibrarySummary>("library_summary");
      warmedGalleryPaths.clear();
      void warmGalleryCache(260);
    } catch (error) {
      errorText = String(error);
    } finally {
      isBusy = false;
    }
  }

  async function focusCurrentFolder() {
    if (isBusy) return;
    isBusy = true;
    errorText = "";
    try {
      const response = await invoke<MediaResponse>("focus_current_folder");
      setCurrentMedia(response);
      summary = await invoke<LibrarySummary>("library_summary");
      warmedGalleryPaths.clear();
      void warmGalleryCache(220);
    } catch (error) {
      errorText = String(error);
    } finally {
      isBusy = false;
    }
  }

  async function backToGlobal() {
    if (isBusy) return;
    isBusy = true;
    errorText = "";
    try {
      const response = await invoke<MediaResponse>("clear_folder_focus");
      setCurrentMedia(response);
      summary = await invoke<LibrarySummary>("library_summary");
      warmedGalleryPaths.clear();
      void warmGalleryCache(260);
    } catch (error) {
      errorText = String(error);
    } finally {
      isBusy = false;
    }
  }

  async function jumpToMedia(path: string) {
    if (isBusy) return;
    isBusy = true;
    errorText = "";
    try {
      const response = await invoke<MediaResponse>("jump_to_media", { path });
      setCurrentMedia(response);
      summary = await invoke<LibrarySummary>("library_summary");
      viewMode = "viewer";
    } catch (error) {
      errorText = String(error);
    } finally {
      isBusy = false;
    }
  }

  async function warmGalleryCache(limit: number) {
    const paths = await invoke<string[]>("gallery_preload_paths", { limit, imagesOnly: true });
    for (const path of paths) {
      if (warmedGalleryPaths.has(path)) continue;
      warmedGalleryPaths.add(path);
      const img = new Image();
      img.decoding = "sync";
      img.src = convertFileSrc(path);
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowRight") {
      void nextMedia();
    } else if (event.key === "ArrowLeft") {
      void previousMedia();
    } else if (event.key === " ") {
      event.preventDefault();
      void nextMedia();
    } else if (event.key.toLowerCase() === "l") {
      void loadLibrary();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class={`h-screen overflow-hidden ${themeClass[themeMode]}`}>
  <div class="grid h-full grid-rows-[auto_minmax(0,1fr)_auto] gap-3 p-3 xl:p-4">
    <header class={`grid gap-2 rounded-2xl border p-3 shadow-sm lg:grid-cols-[230px_230px_minmax(0,1fr)_170px] ${panelClass[themeMode]}`}>
      <Select.Root type="single" bind:value={themeMode}>
        <Select.Trigger class={`w-full rounded-xl border px-3 py-2 text-left text-sm ${controlClass[themeMode]}`}>Theme: {themeMode}</Select.Trigger>
        <Select.Portal>
          <Select.Content class={menuContentClass[themeMode]}>
            <Select.Viewport class="grid gap-1">
              <Select.Item value="light" label="Light" class={menuItemClass[themeMode]}>Light</Select.Item>
              <Select.Item value="dark" label="Dark" class={menuItemClass[themeMode]}>Dark</Select.Item>
              <Select.Item value="mono" label="Mono" class={menuItemClass[themeMode]}>Mono</Select.Item>
              <Select.Item value="vivid" label="Vivid" class={menuItemClass[themeMode]}>Vivid</Select.Item>
              <Select.Item value="ocean" label="Ocean" class={menuItemClass[themeMode]}>Ocean</Select.Item>
              <Select.Item value="sunset" label="Sunset" class={menuItemClass[themeMode]}>Sunset</Select.Item>
              <Select.Item value="aurora" label="Aurora" class={menuItemClass[themeMode]}>Aurora</Select.Item>
              <Select.Item value="carnival" label="Carnival" class={menuItemClass[themeMode]}>Carnival</Select.Item>
            </Select.Viewport>
          </Select.Content>
        </Select.Portal>
      </Select.Root>

      <Select.Root type="single" bind:value={mediaMode}>
        <Select.Trigger class={`w-full rounded-xl border px-3 py-2 text-left text-sm ${controlClass[themeMode]}`}>Media: {mediaModeLabel}</Select.Trigger>
        <Select.Portal>
          <Select.Content class={menuContentClass[themeMode]}>
            <Select.Viewport class="grid gap-1">
              <Select.Item value="images" label="Images only" class={menuItemClass[themeMode]}>Images only</Select.Item>
              <Select.Item value="videos" label="Videos only" class={menuItemClass[themeMode]}>Videos only</Select.Item>
              <Select.Item value="mixed" label="Mixed" class={menuItemClass[themeMode]}>Mixed</Select.Item>
            </Select.Viewport>
          </Select.Content>
        </Select.Portal>
      </Select.Root>

      <div class="min-w-0 overflow-x-auto">
        <Tabs.Root bind:value={viewMode}>
          <Tabs.List class={`inline-flex min-w-max gap-1 rounded-xl p-1 ${softClass[themeMode]}`}>
            <Tabs.Trigger value="viewer" class="rounded-lg px-3 py-1.5 text-sm data-[state=active]:bg-white data-[state=active]:text-slate-900">Viewer</Tabs.Trigger>
            <Tabs.Trigger value="waterfall" class="rounded-lg px-3 py-1.5 text-sm data-[state=active]:bg-white data-[state=active]:text-slate-900">Waterfall</Tabs.Trigger>
            <Tabs.Trigger value="mosaic" class="rounded-lg px-3 py-1.5 text-sm data-[state=active]:bg-white data-[state=active]:text-slate-900">Mosaic</Tabs.Trigger>
          </Tabs.List>
        </Tabs.Root>
      </div>

      <div class={`flex min-h-11 items-center justify-center rounded-xl border px-3 py-2 text-base font-bold tracking-wide ${controlClass[themeMode]}`}>{indexLabel}</div>
    </header>

    <main class="grid min-h-0 grid-cols-1 gap-3 xl:grid-cols-[minmax(0,1fr)_380px]">
      <section class={`grid min-h-0 grid-rows-[minmax(0,1fr)] rounded-2xl border p-3 shadow-sm ${panelClass[themeMode]}`}>
        <div class="min-h-0 max-w-full">
          {#if viewMode === "viewer"}
            <div class={`relative flex h-full min-h-0 max-w-full items-center justify-center overflow-hidden rounded-xl ${softClass[themeMode]}`}>
              {#if currentMedia && currentMediaSrc}
                {#if currentMedia.mediaType === "image"}
                  <img src={currentMediaSrc} alt={currentMedia.fileName} class="max-h-full max-w-full object-contain" draggable="false" />
                {:else}
                  <video class="max-h-full max-w-full object-contain" src={currentMediaSrc} controls preload="metadata" playsinline>
                    <track kind="captions" srclang="en" label="Captions" />
                  </video>
                {/if}
              {:else}
                <div class="max-w-xl px-6 text-center text-sm opacity-80">Load folders to start viewing.</div>
              {/if}
              {#if isBusy}
                <div class={`absolute right-3 top-3 rounded-md px-3 py-1 text-xs font-bold ${accentClass[themeMode]}`}>Loading</div>
              {/if}
            </div>
          {:else if viewMode === "waterfall"}
            <ScrollArea.Root type="always" class={`h-full min-h-0 max-w-full rounded-xl ${softClass[themeMode]}`}>
              <ScrollArea.Viewport class="h-full w-full overflow-x-hidden overflow-y-auto p-3">
                <div class="columns-1 gap-4 sm:columns-2 lg:columns-3 2xl:columns-4">
                  {#each imageCatalog as item, i}
                    <Button.Root
                      class={`mb-4 block w-full break-inside-avoid overflow-hidden text-left shadow-sm transition hover:-translate-y-0.5 hover:shadow-md ${i % 5 === 0 ? "rounded-3xl" : i % 3 === 0 ? "rounded-2xl" : "rounded-xl"} ${i % 2 === 0 ? "bg-black/5 dark:bg-white/10" : "bg-black/10 dark:bg-white/15"}`}
                      onclick={() => jumpToMedia(item.path)}
                    >
                      <img src={convertFileSrc(item.path)} alt={item.fileName} class="h-auto w-full object-cover" loading="lazy" />
                    </Button.Root>
                  {/each}
                </div>
              </ScrollArea.Viewport>
              <ScrollArea.Scrollbar orientation="vertical" class="w-2">
                <ScrollArea.Thumb class="rounded-full bg-black/30 dark:bg-white/30" />
              </ScrollArea.Scrollbar>
            </ScrollArea.Root>
          {:else}
            <ScrollArea.Root type="always" class={`h-full min-h-0 max-w-full rounded-xl ${softClass[themeMode]}`}>
              <ScrollArea.Viewport class="h-full w-full overflow-auto p-3">
                <div class="grid auto-rows-[140px] grid-cols-2 gap-3 md:grid-cols-4 xl:grid-cols-6">
                  {#each imageCatalog as item, i}
                    <Button.Root
                      class={`overflow-hidden rounded-xl bg-black/5 dark:bg-white/10 ${i % 7 === 0 ? "md:col-span-2 md:row-span-2" : i % 5 === 0 ? "md:row-span-2" : ""}`}
                      onclick={() => jumpToMedia(item.path)}
                    >
                      <img src={convertFileSrc(item.path)} alt={item.fileName} class="h-full w-full object-cover" loading="lazy" />
                    </Button.Root>
                  {/each}
                </div>
              </ScrollArea.Viewport>
              <ScrollArea.Scrollbar orientation="vertical" class="w-2">
                <ScrollArea.Thumb class="rounded-full bg-black/30 dark:bg-white/30" />
              </ScrollArea.Scrollbar>
            </ScrollArea.Root>
          {/if}
        </div>
      </section>

      <aside class={`flex min-h-0 flex-col gap-3 rounded-2xl border p-4 shadow-sm ${panelClass[themeMode]}`}>
        <div class="grid grid-cols-2 gap-2">
          <Button.Root class={`rounded-xl border px-3 py-2 text-sm font-semibold ${controlClass[themeMode]}`} onclick={previousMedia} disabled={isBusy || !currentMedia}>Previous</Button.Root>
          <Button.Root class={`rounded-xl border px-3 py-2 text-sm font-semibold ${controlClass[themeMode]}`} onclick={nextMedia} disabled={isBusy || !currentMedia}>Next</Button.Root>
          <Button.Root class={`rounded-xl border px-3 py-2 text-sm font-semibold ${controlClass[themeMode]}`} onclick={reshuffle} disabled={isBusy || !currentMedia}>Shuffle</Button.Root>
          {#if summary.mode === "folder"}
            <Button.Root class={`rounded-xl border px-3 py-2 text-sm font-semibold ${controlClass[themeMode]}`} onclick={backToGlobal} disabled={isBusy || !currentMedia}>Back To Global</Button.Root>
          {:else}
            <Button.Root class={`rounded-xl border px-3 py-2 text-sm font-semibold ${controlClass[themeMode]}`} onclick={focusCurrentFolder} disabled={isBusy || !currentMedia}>This Folder</Button.Root>
          {/if}
        </div>

        <div class="grid grid-cols-1 gap-2">
          <Button.Root class={`rounded-xl border px-3 py-2 text-sm font-semibold ${controlClass[themeMode]}`} onclick={pickIncludeFolder} disabled={isBusy}>Add Include Folder</Button.Root>
          <Button.Root class={`rounded-xl border px-3 py-2 text-sm font-semibold ${controlClass[themeMode]}`} onclick={pickExcludeFolder} disabled={isBusy}>Add Exclude Folder</Button.Root>
        </div>

        <div class={`rounded-xl p-2 ${softClass[themeMode]}`}>
          <div class="mb-2 flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.1em]">
            <span>Presets</span>
            <span class={`rounded-md px-2 py-0.5 text-[10px] ${controlClass[themeMode]}`}>{presets.length}</span>
          </div>
          <div class="grid gap-2">
            <input
              class={`w-full rounded-xl border px-3 py-2 text-sm ${controlClass[themeMode]}`}
              placeholder="Preset name"
              bind:value={presetDraft}
            />

            <Select.Root type="single" bind:value={selectedPreset}>
              <Select.Trigger class={`w-full rounded-xl border px-3 py-2 text-left text-sm ${controlClass[themeMode]}`}>{selectedPreset || "Choose preset"}</Select.Trigger>
              <Select.Portal>
                <Select.Content class={menuContentClass[themeMode]}>
                  <Select.Viewport class="grid gap-1">
                    {#if presets.length === 0}
                      <div class="rounded-lg px-3 py-2 text-sm opacity-70">No presets</div>
                    {:else}
                      {#each presets as preset}
                        <Select.Item value={preset.name} label={preset.name} class={menuItemClass[themeMode]}>{preset.name}</Select.Item>
                      {/each}
                    {/if}
                  </Select.Viewport>
                </Select.Content>
              </Select.Portal>
            </Select.Root>

            <div class="grid grid-cols-4 gap-2">
              <Button.Root class={`rounded-xl border px-3 py-2 text-sm font-semibold ${controlClass[themeMode]}`} onclick={() => selectedPreset && loadPresetByName(selectedPreset)} disabled={!selectedPreset || isBusy}>Load</Button.Root>
              <Button.Root class={`rounded-xl border px-3 py-2 text-sm font-semibold ${controlClass[themeMode]}`} onclick={savePreset}>Save</Button.Root>
              <Button.Root class={`rounded-xl border px-3 py-2 text-sm font-semibold ${controlClass[themeMode]}`} onclick={renamePreset} disabled={!selectedPreset || isBusy}>Rename</Button.Root>
              <Button.Root class={`rounded-xl border px-3 py-2 text-sm font-semibold ${controlClass[themeMode]}`} onclick={deletePreset} disabled={!selectedPreset || isBusy}>Delete</Button.Root>
            </div>
          </div>
        </div>

        <div class={`min-h-[130px] rounded-xl p-2 ${softClass[themeMode]}`}>
          <div class="mb-2 text-xs font-semibold uppercase tracking-[0.1em]">Include Folders</div>
          <ScrollArea.Root type="always" class="h-[160px] rounded-lg">
            <ScrollArea.Viewport class="h-full w-full overflow-auto">
              <ul class="grid gap-2 p-1">
                {#if includeFolders.length === 0}
                  <li class={`rounded-lg px-2 py-2 text-xs ${includeCardClass[themeMode]}`}>No include folders</li>
                {:else}
                  {#each includeFolders as folder}
                    <li class={`rounded-lg p-2 text-xs ${includeCardClass[themeMode]}`}>
                      <div class="mb-2 flex items-center gap-2">
                        <span class="truncate">{folder.path}</span>
                        <Button.Root class="ml-auto rounded-md bg-black/10 px-2 py-1 text-[10px] dark:bg-white/20" onclick={() => removeIncludeFolder(folder.path)}>Remove</Button.Root>
                      </div>
                      <div class="flex items-center gap-2">
                        <Button.Root
                          class={`rounded-md border px-2 py-1 text-[10px] ${folder.stars === 0 ? accentClass[themeMode] : controlClass[themeMode]}`}
                          onclick={() => setFolderStars(folder.path, 0)}
                        >
                          Off
                        </Button.Root>
                        <RatingGroup.Root
                          value={folder.stars}
                          max={STAR_MAX}
                          onValueChange={(value) => setFolderStars(folder.path, Math.round(value))}
                          class="flex items-center gap-1"
                        >
                          {#each Array.from({ length: STAR_MAX }) as _, i}
                            <RatingGroup.Item
                              index={i}
                              class={`text-xl leading-none transition ${i < folder.stars ? starActiveClass[themeMode] : starInactiveClass[themeMode]}`}
                            >
                              ★
                            </RatingGroup.Item>
                          {/each}
                        </RatingGroup.Root>
                      </div>
                    </li>
                  {/each}
                {/if}
              </ul>
            </ScrollArea.Viewport>
            <ScrollArea.Scrollbar orientation="vertical" class="w-2">
              <ScrollArea.Thumb class="rounded-full bg-black/30 dark:bg-white/30" />
            </ScrollArea.Scrollbar>
          </ScrollArea.Root>
        </div>

        <div class={`min-h-[90px] rounded-xl p-2 ${softClass[themeMode]}`}>
          <div class="mb-2 text-xs font-semibold uppercase tracking-[0.1em]">Exclude Folders</div>
          <ScrollArea.Root type="always" class="h-[110px] rounded-lg">
            <ScrollArea.Viewport class="h-full w-full overflow-auto">
              <ul class="grid gap-2 p-1">
                {#if excludeFolders.length === 0}
                  <li class={`rounded-lg px-2 py-2 text-xs ${includeCardClass[themeMode]}`}>No exclude folders</li>
                {:else}
                  {#each excludeFolders as folder}
                    <li class={`flex items-center gap-2 rounded-lg p-2 text-xs ${includeCardClass[themeMode]}`}>
                      <span class="truncate">{folder}</span>
                      <Button.Root class="ml-auto rounded-md bg-black/10 px-2 py-1 text-[10px] dark:bg-white/20" onclick={() => removeExcludeFolder(folder)}>Remove</Button.Root>
                    </li>
                  {/each}
                {/if}
              </ul>
            </ScrollArea.Viewport>
            <ScrollArea.Scrollbar orientation="vertical" class="w-2">
              <ScrollArea.Thumb class="rounded-full bg-black/30 dark:bg-white/30" />
            </ScrollArea.Scrollbar>
          </ScrollArea.Root>
        </div>
      </aside>
    </main>

    <footer class={`rounded-2xl border p-3 shadow-sm ${panelClass[themeMode]}`}>
      <div class="grid grid-cols-1 gap-2 text-xs sm:grid-cols-2 lg:grid-cols-6">
        <div class={`rounded-lg px-3 py-2 ${softClass[themeMode]}`}><span class="font-semibold">File</span> {currentMedia?.fileName || "None"}</div>
        <div class={`rounded-lg px-3 py-2 ${softClass[themeMode]}`}><span class="font-semibold">Total</span> {summary.total}</div>
        <div class={`truncate rounded-lg px-3 py-2 ${softClass[themeMode]}`}><span class="font-semibold">Flow</span> {modeLabel}</div>
        <div class={`truncate rounded-lg px-3 py-2 ${softClass[themeMode]}`}><span class="font-semibold">Focus</span> {summary.focusedFolder || "None"}</div>
        <div class={`rounded-lg px-3 py-2 ${softClass[themeMode]}`}><span class="font-semibold">Media</span> {summary.mediaMode.toUpperCase()}</div>
        <div class={`rounded-lg px-3 py-2 ${softClass[themeMode]}`}><span class="font-semibold">Shortcuts</span> Left/Right/Space, L load</div>
      </div>
      {#if errorText}
        <div class="mt-2 rounded-xl bg-rose-100 px-3 py-2 text-xs text-rose-700 dark:bg-rose-950 dark:text-rose-300">{errorText}</div>
      {/if}
    </footer>
  </div>
</div>
