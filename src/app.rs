use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

use crate::fs_tree::{FlatItem, FsNode, NodeKind, rebuild_flat_list};
use crate::scanner::{ScanMsg, start_scan};


#[derive(Debug, Default, Clone)]
pub struct DriveInfo {
    pub mount: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
}

#[derive(Debug, PartialEq)]
pub enum AppMode {
    Scanning,
    Browsing,
    Confirming,
}

pub struct AppState {
    pub drive: DriveInfo,
    pub root_path: PathBuf,
    pub scan_rx: Option<Receiver<ScanMsg>>,
    pub scan_progress: (usize, u64),
    pub root: Option<FsNode>,
    pub flat_list: Vec<FlatItem>,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub selected_paths: HashSet<PathBuf>,
    pub mode: AppMode,
    pub confirm_targets: Vec<(PathBuf, u64)>,
    pub confirm_total_bytes: u64,
    pub confirm_focus_yes: bool,
    pub status_msg: Option<String>,
    pub should_quit: bool,
    pub skipped_total: usize,
}

impl AppState {
    pub fn new(root_path: PathBuf) -> Self {
        let drive = query_drive_info(&root_path);
        let mut state = Self {
            drive,
            root_path: root_path.clone(),
            scan_rx: None,
            scan_progress: (0, 0),
            root: None,
            flat_list: vec![],
            cursor: 0,
            scroll_offset: 0,
            selected_paths: HashSet::new(),
            mode: AppMode::Scanning,
            confirm_targets: vec![],
            confirm_total_bytes: 0,
            confirm_focus_yes: true,
            status_msg: None,
            should_quit: false,
            skipped_total: 0,
        };
        state.start_scan();
        state
    }

    pub fn start_scan(&mut self) {
        let (tx, rx): (Sender<ScanMsg>, Receiver<ScanMsg>) = mpsc::channel();
        self.scan_rx = Some(rx);
        self.mode = AppMode::Scanning;
        self.scan_progress = (0, 0);
        self.skipped_total = 0;
        // Create an empty root node immediately so streaming children can be appended.
        let mut root = FsNode::new_dir(self.root_path.clone());
        root.expanded = true;
        self.root = Some(root);
        self.rebuild_flat();
        start_scan(self.root_path.clone(), tx);
    }

    pub fn poll_scan(&mut self) {
        // Drain all pending messages this tick so the UI doesn't lag behind.
        loop {
            let msg = if let Some(rx) = &self.scan_rx {
                rx.try_recv().ok()
            } else {
                None
            };
            match msg {
                Some(ScanMsg::Update { node, skipped }) => {
                    self.skipped_total += skipped;
                    self.scan_progress.0 += 1;
                    self.scan_progress.1 += node.size;
                    if let Some(root) = &mut self.root {
                        // Keep existing child if already present (e.g. rescan), else push.
                        if let Some(existing) = root.children.iter_mut().find(|c| c.path == node.path) {
                            *existing = node;
                        } else {
                            root.children.push(node);
                        }
                        // Re-sort and recompute root size after each arrival.
                        root.children.sort_by(|a, b| b.size.cmp(&a.size));
                        root.size = root.children.iter().map(|c| c.size).sum();
                    }
                    self.rebuild_flat();
                    // Switch to Browsing as soon as the first result arrives.
                    if self.mode == AppMode::Scanning {
                        self.mode = AppMode::Browsing;
                    }
                }
                Some(ScanMsg::Done { skipped }) => {
                    self.skipped_total += skipped;
                    self.scan_rx = None;
                    self.mode = AppMode::Browsing;
                    if self.skipped_total > 0 {
                        self.status_msg = Some(format!(
                            "{} path(s) skipped (permission denied — run as administrator to scan them)",
                            self.skipped_total
                        ));
                    } else {
                        self.status_msg = None;
                    }
                    break;
                }
                None => break,
            }
        }
    }

    pub fn rebuild_flat(&mut self) {
        if let Some(root) = &self.root {
            self.flat_list = rebuild_flat_list(root);
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            if self.cursor < self.scroll_offset {
                self.scroll_offset = self.cursor;
            }
        }
    }

    pub fn cursor_down(&mut self, visible_height: usize) {
        if self.cursor + 1 < self.flat_list.len() {
            self.cursor += 1;
            if self.cursor >= self.scroll_offset + visible_height {
                self.scroll_offset = self.cursor + 1 - visible_height;
            }
        }
    }

    pub fn page_up(&mut self, visible_height: usize) {
        let half = (visible_height / 2).max(1);
        self.cursor = self.cursor.saturating_sub(half);
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        }
    }

    pub fn page_down(&mut self, visible_height: usize) {
        let half = (visible_height / 2).max(1);
        let max = self.flat_list.len().saturating_sub(1);
        self.cursor = (self.cursor + half).min(max);
        if self.cursor >= self.scroll_offset + visible_height {
            self.scroll_offset = self.cursor + 1 - visible_height;
        }
    }

    pub fn toggle_expand(&mut self) {
        if let Some(item) = self.flat_list.get(self.cursor) {
            let path = item.path.clone();
            let is_dir = item.kind == NodeKind::Directory;
            let has_children = item.has_children;
            if is_dir && has_children {
                if let Some(root) = &mut self.root {
                    if let Some(node) = root.find_mut(&path) {
                        node.expanded = !node.expanded;
                    }
                }
                self.rebuild_flat();
            }
        }
    }

    pub fn collapse_or_parent(&mut self) {
        if let Some(item) = self.flat_list.get(self.cursor) {
            let is_expanded_dir = item.kind == NodeKind::Directory && item.expanded;
            let path = item.path.clone();
            let depth = item.depth;

            if is_expanded_dir {
                if let Some(root) = &mut self.root {
                    if let Some(node) = root.find_mut(&path) {
                        node.expanded = false;
                    }
                }
                self.rebuild_flat();
            } else if depth > 0 {
                // Jump to parent in flat list
                let parent = path.parent().map(|p| p.to_path_buf());
                if let Some(parent_path) = parent {
                    if let Some(idx) = self.flat_list.iter().position(|i| i.path == parent_path) {
                        self.cursor = idx;
                    }
                }
            }
        }
    }

    pub fn toggle_select(&mut self) {
        if let Some(item) = self.flat_list.get(self.cursor) {
            let path = item.path.clone();
            if self.selected_paths.contains(&path) {
                self.selected_paths.remove(&path);
            } else {
                self.selected_paths.insert(path.clone());
            }
            if let Some(root) = &mut self.root {
                if let Some(node) = root.find_mut(&path) {
                    node.selected = !node.selected;
                }
            }
            self.rebuild_flat();
        }
    }

    pub fn select_all(&mut self) {
        if let Some(root) = &mut self.root {
            select_all_nodes(root, &mut self.selected_paths);
        }
        self.rebuild_flat();
    }

    pub fn open_confirm(&mut self) {
        let targets: Vec<(PathBuf, u64)> = if !self.selected_paths.is_empty() {
            self.selected_paths
                .iter()
                .filter_map(|p| {
                    let size = self.root.as_ref()?.find(p).map(|n| n.size)?;
                    Some((p.clone(), size))
                })
                .collect()
        } else if let Some(item) = self.flat_list.get(self.cursor) {
            vec![(item.path.clone(), item.size)]
        } else {
            return;
        };

        if targets.is_empty() {
            return;
        }

        self.confirm_total_bytes = targets.iter().map(|(_, s)| s).sum();
        self.confirm_targets = targets;
        self.confirm_focus_yes = true;
        self.mode = AppMode::Confirming;
    }

    pub fn execute_delete(&mut self) {
        // Sort longest path first so children are deleted before parents
        let mut targets = self.confirm_targets.clone();
        targets.sort_by(|a, b| b.0.as_os_str().len().cmp(&a.0.as_os_str().len()));

        let mut errors: Vec<String> = vec![];
        let mut deleted = 0usize;

        for (path, _) in &targets {
            let result = if path.is_dir() {
                std::fs::remove_dir_all(path)
            } else {
                std::fs::remove_file(path)
            };
            match result {
                Ok(_) => deleted += 1,
                Err(e) => errors.push(format!("{}: {}", path.display(), e)),
            }
        }

        let freed = humansize::format_size(self.confirm_total_bytes, humansize::BINARY);
        if errors.is_empty() {
            self.status_msg = Some(format!("Deleted {} item(s). Freed ~{}.", deleted, freed));
        } else {
            self.status_msg = Some(format!(
                "Deleted {} item(s). {} error(s): {}",
                deleted,
                errors.len(),
                errors.join("; ")
            ));
        }

        self.confirm_targets.clear();
        self.confirm_total_bytes = 0;
        self.selected_paths.clear();
        self.drive = query_drive_info(&self.root_path);
        self.start_scan();
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm_targets.clear();
        self.mode = AppMode::Browsing;
    }
}

fn select_all_nodes(node: &mut FsNode, selected: &mut HashSet<PathBuf>) {
    node.selected = true;
    selected.insert(node.path.clone());
    for child in &mut node.children {
        select_all_nodes(child, selected);
    }
}

pub fn query_drive_info(path: &PathBuf) -> DriveInfo {
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let path_str = path.to_string_lossy().to_lowercase();

    // Find the best matching disk (longest mount point prefix)
    let mut best: Option<DriveInfo> = None;
    let mut best_len = 0usize;

    for disk in &disks {
        let mount = disk.mount_point().to_string_lossy().to_string();
        let mount_lower = mount.to_lowercase();
        if path_str.starts_with(&mount_lower) && mount_lower.len() > best_len {
            best_len = mount_lower.len();
            best = Some(DriveInfo {
                mount,
                total_bytes: disk.total_space(),
                free_bytes: disk.available_space(),
                used_bytes: disk.total_space().saturating_sub(disk.available_space()),
            });
        }
    }

    best.unwrap_or_default()
}
