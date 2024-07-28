use shared::admin_panel::PatchNote;

#[derive(Default)]
pub(crate) struct PatchNoteHolder {
    pub(crate) take: u32,
    pub(crate) skip: u32,
    pub(crate) total: u32,
    pub(crate) patch_notes: Vec<PatchNote>,
}
