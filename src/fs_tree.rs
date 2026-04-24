use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Directory,
    File,
}

#[derive(Debug, Clone)]
pub struct FsNode {
    pub path: PathBuf,
    pub name: String,
    pub kind: NodeKind,
    pub size: u64,
    pub children: Vec<FsNode>,
    pub expanded: bool,
    pub selected: bool,
}

impl FsNode {
    pub fn new_dir(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        Self {
            path,
            name,
            kind: NodeKind::Directory,
            size: 0,
            children: vec![],
            expanded: false,
            selected: false,
        }
    }

    pub fn new_file(path: PathBuf, size: u64) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        Self {
            path,
            name,
            kind: NodeKind::File,
            size,
            children: vec![],
            expanded: false,
            selected: false,
        }
    }

    pub fn sort_by_size_desc(&mut self) {
        self.children.sort_by(|a, b| b.size.cmp(&a.size));
        for child in &mut self.children {
            child.sort_by_size_desc();
        }
    }

    pub fn find_mut(&mut self, path: &PathBuf) -> Option<&mut FsNode> {
        if &self.path == path {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(path) {
                return Some(found);
            }
        }
        None
    }

    pub fn find(&self, path: &PathBuf) -> Option<&FsNode> {
        if &self.path == path {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(path) {
                return Some(found);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct FlatItem {
    pub path: PathBuf,
    pub depth: usize,
    pub name: String,
    pub kind: NodeKind,
    pub size: u64,
    pub parent_size: u64,
    pub expanded: bool,
    pub selected: bool,
    pub has_children: bool,
}

pub fn rebuild_flat_list(root: &FsNode) -> Vec<FlatItem> {
    let mut list = Vec::new();
    let root_size = root.size;
    collect_flat(root, 0, root_size, &mut list);
    list
}

fn collect_flat(node: &FsNode, depth: usize, parent_size: u64, out: &mut Vec<FlatItem>) {
    out.push(FlatItem {
        path: node.path.clone(),
        depth,
        name: node.name.clone(),
        kind: node.kind.clone(),
        size: node.size,
        parent_size,
        expanded: node.expanded,
        selected: node.selected,
        has_children: !node.children.is_empty(),
    });
    if node.expanded {
        for child in &node.children {
            collect_flat(child, depth + 1, node.size, out);
        }
    }
}
