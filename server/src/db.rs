use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;
use std::sync::OnceLock;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error};
use shared::admin_panel::PatchNote;


trait NextId {
    fn next_id(&self) -> Self;
}

impl NextId for u32 {
    fn next_id(&self) -> Self {
        self+1
    }
}

#[derive(Serialize, Deserialize, Default)]
struct PatchNoteHolder<K: NextId+Eq+Hash+Clone, V> {
    next_id: K,
    items: HashMap<K, V>
}

impl<K: NextId+Eq+Hash+Clone+Serialize, V:Serialize> PatchNoteHolder<K, V> {
    fn add(&mut self, item: V) {
        let mut id = self.next_id.next_id();

        std::mem::swap(&mut id, &mut self.next_id);

        self.items.insert(id, item);
    }

    fn dump(&self, path: &str) -> anyhow::Result<()> {
        let Ok(mut file) = std::fs::File::create(path) else {
            return Err(anyhow::anyhow!("Can't open file {path} for write!"));
        };

        file.write_all(
            ron::ser::to_string_pretty(&self, ron::ser::PrettyConfig::default())
                .unwrap()
                .as_bytes(),
        )?;

        Ok(())
    }
}

static INSTANCE: OnceLock<Database> = OnceLock::new();
static DATABASE_DIR: &str = "./database";

#[derive(Default)]
pub struct Database {
    dir: String,
    patch_notes: RwLock<PatchNoteHolder<u32, PatchNote>>,
}

impl Database {
    pub fn instance<'a>() -> &'a Database {
        &INSTANCE
            .get_or_init(|| Self::load(DATABASE_DIR))
    }

    pub async fn info(&self) -> String {
        let v = self.patch_notes.read().await;

        format!("Database:\n\tTotal patchnotes: {}\n", &v.items.len())
    }

    pub async fn add_patch_note(&self, data: String) -> PatchNote {
        let mut holder = self.patch_notes.write().await;

        let patch_note = PatchNote {
            id: holder.next_id,
            data,
        };

        holder.add(patch_note.clone());

        holder.dump(&format!("{}/patchnotes.ron", self.dir)).unwrap();

        patch_note
    }

    pub async fn update_patch_note(&self, id: u32, data: String) {
        let mut holder = self.patch_notes.write().await;

        if let Some(v) = holder.items.get_mut(&id) {
            v.data = data;
        }

        holder.dump(&format!("{}/patchnotes.ron", self.dir)).unwrap();
    }

    pub async fn patch_notes(&self) -> Vec<PatchNote> {
        let holder = self.patch_notes.read().await;

        let mut res: Vec<_> = holder.items.values().cloned().collect();

        res.sort_by(|a, b| a.id.cmp(&b.id));

        res
    }

    fn load(dir: &str) -> Self {
        let patch_notes= if let Ok(file) = std::fs::File::open(format!("./{dir}/patchnotes.ron")) {
             if let Ok(v) = ron::de::from_reader(file) {
                v
            } else {
                error!("Corrupted database file: ./{dir}/patchnotes.ron");

                Default::default()
            }
        } else {
            Default::default()
        };

        Self {
            patch_notes: RwLock::new(patch_notes),
            dir: dir.to_string(),
        }
    }
}
