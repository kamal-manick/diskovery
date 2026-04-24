use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::fs_tree::FsNode;

pub enum ScanMsg {
    /// Sent each time a top-level child finishes scanning.
    Update { node: FsNode, skipped: usize },
    /// Sent once when all top-level children are done.
    Done { skipped: usize },
}

pub fn start_scan(root: PathBuf, tx: Sender<ScanMsg>) {
    std::thread::spawn(move || {
        // Read the immediate children of root synchronously (fast).
        let top_children: Vec<PathBuf> = match std::fs::read_dir(&root) {
            Err(_) => return,
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .collect(),
        };

        let total_skipped = 0usize;

        // Scan each top-level child in parallel, stream results back as they finish.
        // We use a Mutex-wrapped sender so rayon threads can share it.
        use std::sync::{Arc, Mutex};
        let tx = Arc::new(Mutex::new(tx));

        top_children.par_iter().for_each(|child_path| {
            let (node, skipped) = scan_path(child_path);
            let _ = tx.lock().unwrap().send(ScanMsg::Update { node, skipped });
        });

        // total_skipped is only meaningful for the Done message; per-child skips
        // are already sent with each Update. We set 0 here — the receiver
        // accumulates from Updates.
        let _ = tx.lock().unwrap().send(ScanMsg::Done { skipped: total_skipped });
    });
}

/// Scan a single path (file or directory) recursively. Returns (node, skipped_count).
fn scan_path(path: &PathBuf) -> (FsNode, usize) {
    let meta = match std::fs::metadata(path) {
        Err(_) => {
            // Can't stat root of this subtree at all — return empty placeholder.
            return (FsNode::new_dir(path.clone()), 1);
        }
        Ok(m) => m,
    };

    if meta.is_file() {
        return (FsNode::new_file(path.clone(), meta.len()), 0);
    }

    // It's a directory — walk it fully.
    let mut skipped = 0usize;
    let mut entries = Vec::new();

    for result in WalkDir::new(path).follow_links(false) {
        match result {
            Ok(e) => entries.push(e),
            Err(_) => skipped += 1,
        }
    }

    let stats: Vec<(PathBuf, bool, u64)> = entries
        .par_iter()
        .map(|e| {
            let is_dir = e.file_type().is_dir();
            let size = if is_dir {
                0
            } else {
                e.metadata().map(|m| m.len()).unwrap_or(0)
            };
            (e.path().to_path_buf(), is_dir, size)
        })
        .collect();

    let node = build_tree(path, &stats);
    (node, skipped)
}

fn build_tree(root: &PathBuf, stats: &[(PathBuf, bool, u64)]) -> FsNode {
    let mut dir_sizes: HashMap<PathBuf, u64> = HashMap::new();

    for (path, is_dir, size) in stats {
        if *is_dir {
            dir_sizes.entry(path.clone()).or_insert(0);
        } else {
            let mut ancestor = path.parent().map(|p| p.to_path_buf());
            while let Some(dir) = ancestor {
                *dir_sizes.entry(dir.clone()).or_insert(0) += size;
                if &dir == root {
                    break;
                }
                ancestor = dir.parent().map(|p| p.to_path_buf());
            }
        }
    }

    build_node(root, stats, &dir_sizes)
}

fn build_node(
    path: &PathBuf,
    stats: &[(PathBuf, bool, u64)],
    dir_sizes: &HashMap<PathBuf, u64>,
) -> FsNode {
    let mut node = FsNode::new_dir(path.clone());
    node.size = *dir_sizes.get(path).unwrap_or(&0);

    for (child_path, is_dir, file_size) in stats {
        if child_path.parent().map(|p| p == path).unwrap_or(false) {
            if *is_dir {
                let child = build_node(child_path, stats, dir_sizes);
                node.children.push(child);
            } else {
                node.children.push(FsNode::new_file(child_path.clone(), *file_size));
            }
        }
    }

    node.sort_by_size_desc();
    node
}
