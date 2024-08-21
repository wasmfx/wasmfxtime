use crate::runtime::types::TagType;
use crate::{
    store::{StoreData, StoreOpaque, Stored},
    AsContext,
};

/// A WebAssembly `tag`.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)] // here for the C API
pub struct Tag(pub(super) Stored<crate::runtime::vm::ExportTag>);

impl Tag {
    pub(crate) unsafe fn from_wasmtime_tag(
        mut wasmtime_export: crate::runtime::vm::ExportTag,
        store: &mut StoreOpaque,
    ) -> Self {
        use wasmtime_environ::TypeTrace;
        wasmtime_export
            .tag
            .canonicalize_for_runtime_usage(&mut |module_index| {
                crate::runtime::vm::Instance::from_vmctx(wasmtime_export.vmctx, |instance| {
                    instance.engine_type_index(module_index)
                })
            });

        Tag(store.store_data_mut().insert(wasmtime_export))
    }

    pub(crate) fn ty(&self, _store: impl AsContext) -> TagType {
        todo!()
    }

    pub(crate) fn wasmtime_ty<'a>(&self, data: &'a StoreData) -> &'a wasmtime_environ::Tag {
        &data[self.0].tag
    }

    pub(crate) fn vmimport(&self, store: &StoreOpaque) -> crate::runtime::vm::VMTagImport {
        let export = &store[self.0];
        crate::runtime::vm::VMTagImport {
            from: export.definition,
            vmctx: export.vmctx,
        }
    }
}
