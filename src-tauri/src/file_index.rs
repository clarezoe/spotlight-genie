use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const INDEX_TTL_SECS: u64 = 300;
const MAX_SCAN_DEPTH: usize = 2;
const MAX_SCAN_FILES: usize = 12_000;
const MAX_FOLDER_SCAN_MS: u128 = 120;
const MAX_RESULTS: usize = 3;
const MIN_SCORE: i64 = 30;

#[derive(Clone, Debug)]
struct FileEntry {
    name: String,
    path: String,
    parent: String,
}

#[derive(Clone, Debug)]
pub struct FileMatch {
    pub name: String,
    pub path: String,
    pub parent: String,
    pub score: i64,
}

#[derive(Clone, Debug)]
struct FileIndexState {
    folders: Vec<String>,
    entries: Vec<FileEntry>,
    built_at: SystemTime,
}

impl Default for FileIndexState {
    fn default() -> Self {
        Self {
            folders: Vec::new(),
            entries: Vec::new(),
            built_at: UNIX_EPOCH,
        }
    }
}

static FILE_INDEX: OnceLock<Mutex<FileIndexState>> = OnceLock::new();
static REFRESHING: AtomicBool = AtomicBool::new(false);

pub fn search(
    query: &str,
    matcher: &SkimMatcherV2,
    folders: &[String],
    home: &Path,
) -> Vec<FileMatch> {
    if query.len() < 3 {
        return Vec::new();
    }
    maybe_refresh(folders);
    let entries = snapshot_entries();
    let mut matches = Vec::new();
    for entry in entries {
        let Some(score) = matcher.fuzzy_match(&entry.name, query) else {
            continue;
        };
        if score < MIN_SCORE {
            continue;
        }
        matches.push(FileMatch {
            name: entry.name,
            path: entry.path,
            parent: display_parent(&entry.parent, home),
            score,
        });
    }
    matches.sort_by(|a, b| b.score.cmp(&a.score));
    matches.truncate(MAX_RESULTS);
    matches
}

fn snapshot_entries() -> Vec<FileEntry> {
    let lock = FILE_INDEX.get_or_init(|| Mutex::new(FileIndexState::default()));
    lock.lock()
        .map(|state| state.entries.clone())
        .unwrap_or_default()
}

fn maybe_refresh(folders: &[String]) {
    if !needs_refresh(folders) || REFRESHING.swap(true, Ordering::SeqCst) {
        return;
    }
    let folders_owned = folders.to_vec();
    thread::spawn(move || {
        let entries = build_index(&folders_owned);
        let lock = FILE_INDEX.get_or_init(|| Mutex::new(FileIndexState::default()));
        if let Ok(mut state) = lock.lock() {
            state.folders = folders_owned;
            state.entries = entries;
            state.built_at = SystemTime::now();
        }
        REFRESHING.store(false, Ordering::SeqCst);
    });
}

fn needs_refresh(folders: &[String]) -> bool {
    let lock = FILE_INDEX.get_or_init(|| Mutex::new(FileIndexState::default()));
    let Ok(state) = lock.lock() else {
        return false;
    };
    if state.entries.is_empty() || state.folders != folders {
        return true;
    }
    state
        .built_at
        .elapsed()
        .map(|age| age > Duration::from_secs(INDEX_TTL_SECS))
        .unwrap_or(true)
}

fn build_index(folders: &[String]) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    for folder in folders {
        let root = PathBuf::from(folder);
        if !root.exists() {
            continue;
        }
        collect_folder_entries(&root, &mut entries);
        if entries.len() >= MAX_SCAN_FILES {
            break;
        }
    }
    entries
}

fn collect_folder_entries(root: &Path, entries: &mut Vec<FileEntry>) {
    let started = Instant::now();
    for item in walkdir::WalkDir::new(root)
        .max_depth(MAX_SCAN_DEPTH)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entries.len() >= MAX_SCAN_FILES || started.elapsed().as_millis() > MAX_FOLDER_SCAN_MS {
            break;
        }
        let path = item.path();
        if is_hidden(path) {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let parent = path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        entries.push(FileEntry {
            name: name.to_string(),
            path: path.to_string_lossy().to_string(),
            parent,
        });
    }
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(true)
}

fn display_parent(parent: &str, home: &Path) -> String {
    let path = Path::new(parent);
    path.strip_prefix(home)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}
