use crate::store::{AutoAssertNoGc, StoreOpaque};
use crate::{GlobalType, HeapType, Mutability, Result, Val};
use std::ptr;
use wasmtime_runtime::{StoreBox, VMGlobalDefinition};

#[repr(C)]
pub struct VMHostGlobalContext {
    pub(crate) ty: GlobalType,
    pub(crate) global: VMGlobalDefinition,
}

impl Drop for VMHostGlobalContext {
    fn drop(&mut self) {
        match self.ty.content() {
            crate::ValType::I32
            | crate::ValType::I64
            | crate::ValType::F32
            | crate::ValType::F64
            | crate::ValType::V128 => {
                // Nothing to drop.
            }
            crate::ValType::Ref(r) => match r.heap_type() {
                HeapType::Func | HeapType::Concrete(_) | HeapType::NoFunc => {
                    // Nothing to drop.
                }
                HeapType::Cont | HeapType::NoCont => {
                    // We may have to drop the dynamic continuation Xobject here.
                    todo!("Drop for VMHostGlobalContext with content of type HeapType::Cont and HeapType::NoCont not yet implemented")
                }
                HeapType::Extern => unsafe { ptr::drop_in_place(self.global.as_externref_mut()) },
            },
        }
    }
}

pub fn generate_global_export(
    store: &mut StoreOpaque,
    ty: GlobalType,
    val: Val,
) -> Result<wasmtime_runtime::ExportGlobal> {
    let global = wasmtime_environ::Global {
        wasm_ty: ty.content().to_wasm_type(),
        mutability: match ty.mutability() {
            Mutability::Const => false,
            Mutability::Var => true,
        },
    };
    let ctx = StoreBox::new(VMHostGlobalContext {
        ty,
        global: VMGlobalDefinition::new(),
    });

    let mut store = AutoAssertNoGc::new(store);
    let definition = unsafe {
        let global = &mut (*ctx.get()).global;
        match val {
            Val::I32(x) => *global.as_i32_mut() = x,
            Val::I64(x) => *global.as_i64_mut() = x,
            Val::F32(x) => *global.as_f32_bits_mut() = x,
            Val::F64(x) => *global.as_f64_bits_mut() = x,
            Val::V128(x) => *global.as_u128_mut() = x.into(),
            Val::FuncRef(f) => {
                *global.as_func_ref_mut() =
                    f.map_or(ptr::null_mut(), |f| f.vm_func_ref(&mut store).as_ptr());
            }
            Val::ExternRef(x) => {
                *global.as_externref_mut() = match x {
                    None => None,
                    Some(x) => Some(x.try_to_vm_extern_ref(&mut store)?),
                };
            }
        }
        global
    };

    store.host_globals().push(ctx);
    Ok(wasmtime_runtime::ExportGlobal { definition, global })
}
