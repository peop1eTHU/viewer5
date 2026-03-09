use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use walkdir::WalkDir;

const SUPPORTED_IMAGE_EXTENSIONS: [&str; 10] = [
    "jpg", "jpeg", "png", "webp", "bmp", "gif", "tif", "tiff", "avif", "heic",
];

const SUPPORTED_VIDEO_EXTENSIONS: [&str; 10] = [
    "mp4", "mov", "webm", "mkv", "avi", "m4v", "mpg", "mpeg", "wmv", "flv",
];

const PRELOAD_AHEAD: usize = 28;
const DEFAULT_GALLERY_PRELOAD: usize = 220;

struct AppState(Mutex<LibraryState>);

impl Default for AppState {
    fn default() -> Self {
        Self(Mutex::new(LibraryState::new()))
    }
}

#[derive(Clone)]
enum NavigationMode {
    Global { pos: usize },
    Folder { folder: PathBuf, pos: usize },
}

#[derive(Clone, Copy)]
enum MediaKind {
    Image,
    Video,
}

impl MediaKind {
    fn as_str(self) -> &'static str {
        match self {
            MediaKind::Image => "image",
            MediaKind::Video => "video",
        }
    }
}

#[derive(Clone, Copy)]
enum MediaMode {
    Images,
    Videos,
    Mixed,
}

impl MediaMode {
    fn from_input(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "images" | "image" => Self::Images,
            "videos" | "video" => Self::Videos,
            _ => Self::Mixed,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            MediaMode::Images => "images",
            MediaMode::Videos => "videos",
            MediaMode::Mixed => "mixed",
        }
    }

    fn allows(self, kind: MediaKind) -> bool {
        match self {
            MediaMode::Mixed => true,
            MediaMode::Images => matches!(kind, MediaKind::Image),
            MediaMode::Videos => matches!(kind, MediaKind::Video),
        }
    }
}

#[derive(Clone)]
struct IncludeFolder {
    path: PathBuf,
    stars: u8,
}

#[derive(Clone)]
struct MediaItem {
    path: PathBuf,
    kind: MediaKind,
    include_index: usize,
}

struct LibraryState {
    include_folders: Vec<IncludeFolder>,
    exclude_folders: Vec<PathBuf>,
    media_items: Vec<MediaItem>,
    global_order: Vec<usize>,
    folder_groups: HashMap<PathBuf, Vec<usize>>,
    mode: NavigationMode,
    media_mode: MediaMode,
}

impl LibraryState {
    fn new() -> Self {
        Self {
            include_folders: Vec::new(),
            exclude_folders: Vec::new(),
            media_items: Vec::new(),
            global_order: Vec::new(),
            folder_groups: HashMap::new(),
            mode: NavigationMode::Global { pos: 0 },
            media_mode: MediaMode::Mixed,
        }
    }

    fn set_library(
        &mut self,
        include_folders: Vec<IncludeFolder>,
        exclude_folders: Vec<PathBuf>,
        media_items: Vec<MediaItem>,
        media_mode: MediaMode,
    ) {
        self.include_folders = include_folders;
        self.exclude_folders = exclude_folders;
        self.media_items = media_items;
        self.folder_groups = build_folder_groups(&self.media_items);
        self.media_mode = media_mode;
        self.mode = NavigationMode::Global { pos: 0 };
        self.reshuffle_global();
    }

    fn has_media(&self) -> bool {
        !self.media_items.is_empty()
    }

    fn total_global(&self) -> usize {
        self.media_items.len()
    }

    fn current_media_index(&self) -> Option<usize> {
        match &self.mode {
            NavigationMode::Global { pos } => self.global_order.get(*pos).copied(),
            NavigationMode::Folder { folder, pos } => self
                .folder_groups
                .get(folder)
                .and_then(|group| group.get(*pos).copied()),
        }
    }

    fn current_view_total(&self) -> usize {
        match &self.mode {
            NavigationMode::Global { .. } => self.global_order.len(),
            NavigationMode::Folder { folder, .. } => self.folder_groups.get(folder).map_or(0, Vec::len),
        }
    }

    fn current_view_position(&self) -> usize {
        match &self.mode {
            NavigationMode::Global { pos } => *pos,
            NavigationMode::Folder { pos, .. } => *pos,
        }
    }

    fn mode_name(&self) -> &'static str {
        match self.mode {
            NavigationMode::Global { .. } => "global",
            NavigationMode::Folder { .. } => "folder",
        }
    }

    fn focused_folder_label(&self) -> String {
        match &self.mode {
            NavigationMode::Folder { folder, .. } => folder.to_string_lossy().to_string(),
            NavigationMode::Global { .. } => String::new(),
        }
    }

    fn move_next(&mut self) {
        if !self.has_media() {
            return;
        }

        match &mut self.mode {
            NavigationMode::Global { pos } => {
                let total = self.global_order.len();
                if total > 0 {
                    *pos = (*pos + 1) % total;
                }
            }
            NavigationMode::Folder { folder, pos } => {
                if let Some(group) = self.folder_groups.get(folder) {
                    let total = group.len();
                    if total > 0 {
                        *pos = (*pos + 1) % total;
                    }
                }
            }
        }
    }

    fn move_previous(&mut self) {
        if !self.has_media() {
            return;
        }

        match &mut self.mode {
            NavigationMode::Global { pos } => {
                let total = self.global_order.len();
                if total > 0 {
                    *pos = (*pos + total - 1) % total;
                }
            }
            NavigationMode::Folder { folder, pos } => {
                if let Some(group) = self.folder_groups.get(folder) {
                    let total = group.len();
                    if total > 0 {
                        *pos = (*pos + total - 1) % total;
                    }
                }
            }
        }
    }

    fn reshuffle_global(&mut self) {
        self.global_order = build_weighted_order(&self.media_items, &self.include_folders);
        self.mode = NavigationMode::Global { pos: 0 };
    }

    fn focus_current_folder(&mut self) -> Result<(), String> {
        let current_idx = self
            .current_media_index()
            .ok_or_else(|| "No media library is loaded.".to_string())?;

        let folder = self.media_items[current_idx]
            .path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "The current media item has no parent folder.".to_string())?;

        let group = self
            .folder_groups
            .get(&folder)
            .ok_or_else(|| "The current folder has no indexed media.".to_string())?;

        let folder_pos = group.iter().position(|idx| *idx == current_idx).unwrap_or(0);
        self.mode = NavigationMode::Folder {
            folder,
            pos: folder_pos,
        };

        Ok(())
    }

    fn clear_folder_focus(&mut self) {
        let current_idx = match self.current_media_index() {
            Some(idx) => idx,
            None => {
                self.mode = NavigationMode::Global { pos: 0 };
                return;
            }
        };

        let global_pos = self
            .global_order
            .iter()
            .position(|idx| *idx == current_idx)
            .unwrap_or(0);

        self.mode = NavigationMode::Global { pos: global_pos };
    }

    fn preload_candidates(&self, count: usize) -> Vec<PathBuf> {
        let total = self.current_view_total();
        if total <= 1 || count == 0 {
            return Vec::new();
        }

        let current_pos = self.current_view_position();
        let max_count = count.min(total - 1);
        let mut result = Vec::with_capacity(max_count);

        for offset in 1..=max_count {
            let next_pos = (current_pos + offset) % total;
            let maybe_idx = match &self.mode {
                NavigationMode::Global { .. } => self.global_order.get(next_pos).copied(),
                NavigationMode::Folder { folder, .. } => self
                    .folder_groups
                    .get(folder)
                    .and_then(|group| group.get(next_pos).copied()),
            };

            if let Some(idx) = maybe_idx {
                if let Some(item) = self.media_items.get(idx) {
                    result.push(item.path.clone());
                }
            }
        }

        result
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncludeFolderInput {
    path: String,
    stars: u8,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct IncludeFolderSummary {
    path: String,
    stars: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaResponse {
    path: String,
    file_name: String,
    media_type: String,
    preload_paths: Vec<String>,
    index: usize,
    total: usize,
    mode: String,
    focused_folder: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaCatalogItem {
    path: String,
    file_name: String,
    media_type: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PresetRecord {
    name: String,
    include_folders: Vec<IncludeFolderSummary>,
    exclude_folders: Vec<String>,
    media_mode: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LibrarySummary {
    total: usize,
    include_folders: Vec<IncludeFolderSummary>,
    exclude_folders: Vec<String>,
    mode: String,
    focused_folder: String,
    media_mode: String,
}

#[tauri::command]
async fn build_library(
    include_folders: Vec<IncludeFolderInput>,
    exclude_folders: Vec<String>,
    media_mode: String,
    state: State<'_, AppState>,
) -> Result<MediaResponse, String> {
    if include_folders.is_empty() {
        return Err("Add at least one include folder before loading.".to_string());
    }

    let includes = normalize_include_folders(include_folders)?;
    let excludes = normalize_exclude_folders(exclude_folders)?;
    let selected_media_mode = MediaMode::from_input(&media_mode);

    let active_includes = includes
        .iter()
        .filter(|folder| folder.stars > 0)
        .cloned()
        .collect::<Vec<_>>();

    if active_includes.is_empty() {
        return Err("At least one include folder must have 1-5 stars.".to_string());
    }

    let media_items = scan_media(&active_includes, &excludes, selected_media_mode)?;
    if media_items.is_empty() {
        return Err("No matching media files were found for the selected mode.".to_string());
    }

    {
        let mut guard = lock_library(&state)?;
        guard.set_library(active_includes, excludes, media_items, selected_media_mode);
    }

    read_current_media_and_collect_preload(&state)
}

#[tauri::command]
async fn next_image(state: State<'_, AppState>) -> Result<MediaResponse, String> {
    {
        let mut guard = lock_library(&state)?;
        if !guard.has_media() {
            return Err("No media library is loaded.".to_string());
        }
        guard.move_next();
    }

    read_current_media_and_collect_preload(&state)
}

#[tauri::command]
async fn previous_image(state: State<'_, AppState>) -> Result<MediaResponse, String> {
    {
        let mut guard = lock_library(&state)?;
        if !guard.has_media() {
            return Err("No media library is loaded.".to_string());
        }
        guard.move_previous();
    }

    read_current_media_and_collect_preload(&state)
}

#[tauri::command]
async fn reshuffle_library(state: State<'_, AppState>) -> Result<MediaResponse, String> {
    {
        let mut guard = lock_library(&state)?;
        if !guard.has_media() {
            return Err("No media library is loaded.".to_string());
        }
        guard.reshuffle_global();
    }

    read_current_media_and_collect_preload(&state)
}

#[tauri::command]
async fn focus_current_folder(state: State<'_, AppState>) -> Result<MediaResponse, String> {
    {
        let mut guard = lock_library(&state)?;
        if !guard.has_media() {
            return Err("No media library is loaded.".to_string());
        }
        guard.focus_current_folder()?;
    }

    read_current_media_and_collect_preload(&state)
}

#[tauri::command]
async fn clear_folder_focus(state: State<'_, AppState>) -> Result<MediaResponse, String> {
    {
        let mut guard = lock_library(&state)?;
        if !guard.has_media() {
            return Err("No media library is loaded.".to_string());
        }
        guard.clear_folder_focus();
    }

    read_current_media_and_collect_preload(&state)
}

#[tauri::command]
async fn media_catalog(state: State<'_, AppState>) -> Result<Vec<MediaCatalogItem>, String> {
    let guard = lock_library(&state)?;
    Ok(guard
        .media_items
        .iter()
        .map(|item| MediaCatalogItem {
            path: item.path.to_string_lossy().to_string(),
            file_name: item
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string(),
            media_type: item.kind.as_str().to_string(),
        })
        .collect())
}

#[tauri::command]
async fn gallery_preload_paths(
    limit: Option<usize>,
    images_only: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let guard = lock_library(&state)?;
    if !guard.has_media() {
        return Ok(Vec::new());
    }

    let current_pos = guard.current_view_position();
    let total = guard.current_view_total();
    if total == 0 {
        return Ok(Vec::new());
    }

    let target = limit.unwrap_or(DEFAULT_GALLERY_PRELOAD).max(32).min(1000);
    let only_images = images_only.unwrap_or(true);

    let mut result = Vec::with_capacity(target);
    for offset in 0..total {
        if result.len() >= target {
            break;
        }

        let pos = (current_pos + offset) % total;
        let maybe_idx = match &guard.mode {
            NavigationMode::Global { .. } => guard.global_order.get(pos).copied(),
            NavigationMode::Folder { folder, .. } => guard
                .folder_groups
                .get(folder)
                .and_then(|group| group.get(pos).copied()),
        };

        let Some(idx) = maybe_idx else {
            continue;
        };
        let Some(item) = guard.media_items.get(idx) else {
            continue;
        };

        if only_images && !matches!(item.kind, MediaKind::Image) {
            continue;
        }

        result.push(item.path.to_string_lossy().to_string());
    }

    Ok(result)
}

#[tauri::command]
async fn gallery_ordered_catalog(state: State<'_, AppState>) -> Result<Vec<MediaCatalogItem>, String> {
    let guard = lock_library(&state)?;
    if !guard.has_media() {
        return Ok(Vec::new());
    }

    let total = guard.current_view_total();
    let mut result = Vec::with_capacity(total);

    for pos in 0..total {
        let maybe_idx = match &guard.mode {
            NavigationMode::Global { .. } => guard.global_order.get(pos).copied(),
            NavigationMode::Folder { folder, .. } => guard
                .folder_groups
                .get(folder)
                .and_then(|group| group.get(pos).copied()),
        };

        let Some(idx) = maybe_idx else {
            continue;
        };
        let Some(item) = guard.media_items.get(idx) else {
            continue;
        };

        result.push(MediaCatalogItem {
            path: item.path.to_string_lossy().to_string(),
            file_name: item
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string(),
            media_type: item.kind.as_str().to_string(),
        });
    }

    Ok(result)
}

#[tauri::command]
async fn list_presets(app: AppHandle) -> Result<Vec<PresetRecord>, String> {
    read_presets(&app)
}

#[tauri::command]
async fn save_preset(
    app: AppHandle,
    name: String,
    include_folders: Vec<IncludeFolderInput>,
    exclude_folders: Vec<String>,
    media_mode: String,
) -> Result<Vec<PresetRecord>, String> {
    let normalized_name = name.trim().to_string();
    if normalized_name.is_empty() {
        return Err("Preset name cannot be empty.".to_string());
    }

    let includes = normalize_include_folders(include_folders)?;
    let excludes = normalize_exclude_folders(exclude_folders)?;

    let record = PresetRecord {
        name: normalized_name,
        include_folders: includes
            .iter()
            .map(|folder| IncludeFolderSummary {
                path: folder.path.to_string_lossy().to_string(),
                stars: folder.stars,
            })
            .collect(),
        exclude_folders: excludes
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        media_mode,
    };

    let mut presets = read_presets(&app)?;
    if let Some(existing) = presets.iter_mut().find(|preset| preset.name == record.name) {
        *existing = record;
    } else {
        presets.push(record);
    }

    presets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    write_presets(&app, &presets)?;
    Ok(presets)
}

#[tauri::command]
async fn delete_preset(app: AppHandle, name: String) -> Result<Vec<PresetRecord>, String> {
    let normalized_name = name.trim();
    if normalized_name.is_empty() {
        return Err("Preset name cannot be empty.".to_string());
    }

    let mut presets = read_presets(&app)?;
    presets.retain(|preset| preset.name != normalized_name);
    write_presets(&app, &presets)?;
    Ok(presets)
}

#[tauri::command]
async fn rename_preset(app: AppHandle, old_name: String, new_name: String) -> Result<Vec<PresetRecord>, String> {
    let old_name = old_name.trim();
    let new_name = new_name.trim();

    if old_name.is_empty() || new_name.is_empty() {
        return Err("Preset name cannot be empty.".to_string());
    }

    let mut presets = read_presets(&app)?;
    if presets.iter().any(|preset| preset.name == new_name) {
        return Err("A preset with this name already exists.".to_string());
    }

    let Some(target) = presets.iter_mut().find(|preset| preset.name == old_name) else {
        return Err("Preset not found.".to_string());
    };

    target.name = new_name.to_string();
    presets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    write_presets(&app, &presets)?;
    Ok(presets)
}

#[tauri::command]
async fn load_preset(app: AppHandle, name: String) -> Result<PresetRecord, String> {
    let normalized_name = name.trim();
    if normalized_name.is_empty() {
        return Err("Preset name cannot be empty.".to_string());
    }

    let presets = read_presets(&app)?;
    presets
        .into_iter()
        .find(|preset| preset.name == normalized_name)
        .ok_or_else(|| "Preset not found.".to_string())
}

#[tauri::command]
async fn jump_to_media(path: String, state: State<'_, AppState>) -> Result<MediaResponse, String> {
    {
        let mut guard = lock_library(&state)?;
        if !guard.has_media() {
            return Err("No media library is loaded.".to_string());
        }

        let normalized_target = std::fs::canonicalize(PathBuf::from(path)).map_err(|_| {
            "The selected media path could not be resolved on disk.".to_string()
        })?;

        let maybe_media_index = guard
            .media_items
            .iter()
            .position(|item| item.path == normalized_target)
            .ok_or_else(|| "The selected media item is not in the current library.".to_string())?;

        let global_pos = guard
            .global_order
            .iter()
            .position(|idx| *idx == maybe_media_index)
            .ok_or_else(|| "The selected media item could not be mapped in global order.".to_string())?;

        guard.mode = NavigationMode::Global { pos: global_pos };
    }

    read_current_media_and_collect_preload(&state)
}

#[tauri::command]
async fn library_summary(state: State<'_, AppState>) -> Result<LibrarySummary, String> {
    let guard = lock_library(&state)?;
    Ok(LibrarySummary {
        total: guard.total_global(),
        include_folders: guard
            .include_folders
            .iter()
            .map(|folder| IncludeFolderSummary {
                path: folder.path.to_string_lossy().to_string(),
                stars: folder.stars,
            })
            .collect(),
        exclude_folders: guard
            .exclude_folders
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        mode: guard.mode_name().to_string(),
        focused_folder: guard.focused_folder_label(),
        media_mode: guard.media_mode.as_str().to_string(),
    })
}

fn lock_library<'a>(state: &'a State<'_, AppState>) -> Result<std::sync::MutexGuard<'a, LibraryState>, String> {
    state
        .0
        .lock()
        .map_err(|_| "The application state is temporarily unavailable.".to_string())
}

fn preset_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let mut dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("Failed to resolve app data path: {err}"))?;
    fs::create_dir_all(&dir)
        .map_err(|err| format!("Failed to create app data directory: {err}"))?;
    dir.push("presets.json");
    Ok(dir)
}

fn read_presets(app: &AppHandle) -> Result<Vec<PresetRecord>, String> {
    let file_path = preset_file_path(app)?;
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(&file_path)
        .map_err(|err| format!("Failed to read presets file: {err}"))?;

    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str::<Vec<PresetRecord>>(&text)
        .map_err(|err| format!("Failed to parse presets file: {err}"))
}

fn write_presets(app: &AppHandle, presets: &[PresetRecord]) -> Result<(), String> {
    let file_path = preset_file_path(app)?;
    let payload = serde_json::to_string_pretty(presets)
        .map_err(|err| format!("Failed to serialize presets: {err}"))?;
    fs::write(&file_path, payload)
        .map_err(|err| format!("Failed to write presets file: {err}"))
}

fn normalize_include_folders(raw: Vec<IncludeFolderInput>) -> Result<Vec<IncludeFolder>, String> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();

    for entry in raw {
        let path = PathBuf::from(entry.path);
        if !path.exists() || !path.is_dir() {
            continue;
        }

        let normalized = std::fs::canonicalize(&path).unwrap_or(path);
        let key = normalized.to_string_lossy().to_string();
        if seen.insert(key) {
            result.push(IncludeFolder {
                path: normalized,
                stars: entry.stars.min(5),
            });
        }
    }

    if result.is_empty() {
        return Err("No valid include folders are available.".to_string());
    }

    Ok(result)
}

fn normalize_exclude_folders(raw: Vec<String>) -> Result<Vec<PathBuf>, String> {
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();

    for value in raw {
        let path = PathBuf::from(value);
        if !path.exists() || !path.is_dir() {
            continue;
        }

        let normalized = std::fs::canonicalize(&path).unwrap_or(path);
        let key = normalized.to_string_lossy().to_string();
        if seen.insert(key) {
            result.push(normalized);
        }
    }

    Ok(result)
}

fn detect_media_kind(path: &Path) -> Option<MediaKind> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())?;

    if SUPPORTED_IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        return Some(MediaKind::Image);
    }

    if SUPPORTED_VIDEO_EXTENSIONS.contains(&extension.as_str()) {
        return Some(MediaKind::Video);
    }

    None
}

fn scan_media(
    include_folders: &[IncludeFolder],
    exclude_folders: &[PathBuf],
    media_mode: MediaMode,
) -> Result<Vec<MediaItem>, String> {
    let mut unique = BTreeSet::new();
    let mut media_items = Vec::new();

    for (include_index, root) in include_folders.iter().enumerate() {
        for entry in WalkDir::new(&root.path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !is_excluded(entry.path(), exclude_folders))
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if !path.is_file() || is_excluded(path, exclude_folders) {
                continue;
            }

            let Some(kind) = detect_media_kind(path) else {
                continue;
            };

            if !media_mode.allows(kind) {
                continue;
            }

            let normalized = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
            let key = normalized.to_string_lossy().to_string();
            if unique.insert(key) {
                media_items.push(MediaItem {
                    path: normalized,
                    kind,
                    include_index,
                });
            }
        }
    }

    media_items.sort_by_key(|item| item.path.clone());
    Ok(media_items)
}

fn is_excluded(path: &Path, excludes: &[PathBuf]) -> bool {
    excludes.iter().any(|excluded| path.starts_with(excluded))
}

fn build_folder_groups(media_items: &[MediaItem]) -> HashMap<PathBuf, Vec<usize>> {
    let mut groups = HashMap::<PathBuf, Vec<usize>>::new();

    for (idx, item) in media_items.iter().enumerate() {
        let folder = item.path.parent().map_or_else(PathBuf::new, Path::to_path_buf);
        groups.entry(folder).or_default().push(idx);
    }

    for group in groups.values_mut() {
        group.sort_by_key(|idx| media_items[*idx].path.clone());
    }

    groups
}

fn build_weighted_order(media_items: &[MediaItem], include_folders: &[IncludeFolder]) -> Vec<usize> {
    let mut scored = media_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let stars = include_folders
                .get(item.include_index)
                .map(|folder| folder.stars)
                .unwrap_or(1)
                .max(1);
            let weight = stars as f64;
            let random = rand::random::<f64>().clamp(f64::MIN_POSITIVE, 1.0);
            let score = -random.ln() / weight;
            (score, idx)
        })
        .collect::<Vec<_>>();

    scored.sort_by(|left, right| match left.0.partial_cmp(&right.0) {
        Some(ordering) => ordering,
        None => Ordering::Equal,
    });

    scored.into_iter().map(|(_, idx)| idx).collect()
}

fn read_current_media_and_collect_preload(state: &State<'_, AppState>) -> Result<MediaResponse, String> {
    let (item, index, total, preload_paths, mode, focused_folder) = {
        let guard = lock_library(state)?;
        let current_idx = guard
            .current_media_index()
            .ok_or_else(|| "No media library is loaded.".to_string())?;

        let item = guard
            .media_items
            .get(current_idx)
            .cloned()
            .ok_or_else(|| "No media library is loaded.".to_string())?;

        let index = guard.current_view_position();
        let total = guard.current_view_total();
        let preload_paths = guard.preload_candidates(PRELOAD_AHEAD);
        let mode = guard.mode_name().to_string();
        let focused_folder = guard.focused_folder_label();

        (item, index, total, preload_paths, mode, focused_folder)
    };

    let file_name = item
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();

    Ok(MediaResponse {
        path: item.path.to_string_lossy().to_string(),
        file_name,
        media_type: item.kind.as_str().to_string(),
        preload_paths: preload_paths
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect(),
        index,
        total,
        mode,
        focused_folder,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            build_library,
            next_image,
            previous_image,
            reshuffle_library,
            focus_current_folder,
            clear_folder_focus,
            library_summary,
            media_catalog,
            jump_to_media,
            gallery_preload_paths,
            gallery_ordered_catalog,
            list_presets,
            save_preset,
            rename_preset,
            delete_preset,
            load_preset
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
