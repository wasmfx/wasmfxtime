//! ISLE integration glue code for Pulley lowering.

// Pull in the ISLE generated code.
pub mod generated_code;
use generated_code::MInst;
use inst::InstAndKind;

// Types that the generated ISLE code uses via `use super::*`.
use crate::ir::{condcodes::*, immediates::*, types::*, *};
use crate::isa::pulley_shared::{
    abi::*,
    inst::{FReg, VReg, WritableFReg, WritableVReg, WritableXReg, XReg},
    *,
};
use crate::machinst::{
    abi::{ArgPair, RetPair, StackAMode},
    isle::*,
    CallInfo, IsTailCall, MachInst, Reg, VCodeConstant, VCodeConstantData,
};
use alloc::boxed::Box;
use regalloc2::PReg;
type Unit = ();
type VecArgPair = Vec<ArgPair>;
type VecRetPair = Vec<RetPair>;
type BoxCallInfo = Box<CallInfo<ExternalName>>;
type BoxCallIndInfo = Box<CallInfo<XReg>>;
type BoxExternalName = Box<ExternalName>;

pub(crate) struct PulleyIsleContext<'a, 'b, I, B>
where
    I: VCodeInst,
    B: LowerBackend,
{
    pub lower_ctx: &'a mut Lower<'b, I>,
    pub backend: &'a B,
}

impl<'a, 'b, P> PulleyIsleContext<'a, 'b, InstAndKind<P>, PulleyBackend<P>>
where
    P: PulleyTargetKind,
{
    fn new(lower_ctx: &'a mut Lower<'b, InstAndKind<P>>, backend: &'a PulleyBackend<P>) -> Self {
        Self { lower_ctx, backend }
    }
}

impl<P> generated_code::Context for PulleyIsleContext<'_, '_, InstAndKind<P>, PulleyBackend<P>>
where
    P: PulleyTargetKind,
{
    crate::isle_lower_prelude_methods!(InstAndKind<P>);
    crate::isle_prelude_caller_methods!(PulleyMachineDeps<P>, PulleyABICallSite<P>);

    fn gen_return_call(
        &mut self,
        callee_sig: SigRef,
        callee: ExternalName,
        distance: RelocDistance,
        args: ValueSlice,
    ) -> InstOutput {
        let caller_conv = isa::CallConv::Tail;
        debug_assert_eq!(
            self.lower_ctx.abi().call_conv(self.lower_ctx.sigs()),
            caller_conv,
            "Can only do `return_call`s from within a `tail` calling convention function"
        );

        let call_site = PulleyABICallSite::from_func(
            self.lower_ctx.sigs(),
            callee_sig,
            &callee,
            IsTailCall::Yes,
            distance,
            caller_conv,
            self.backend.flags().clone(),
        );
        call_site.emit_return_call(self.lower_ctx, args);

        InstOutput::new()
    }

    fn gen_return_call_indirect(
        &mut self,
        callee_sig: SigRef,
        callee: Value,
        args: ValueSlice,
    ) -> InstOutput {
        let caller_conv = isa::CallConv::Tail;
        debug_assert_eq!(
            self.lower_ctx.abi().call_conv(self.lower_ctx.sigs()),
            caller_conv,
            "Can only do `return_call`s from within a `tail` calling convention function"
        );

        let callee = self.put_in_reg(callee);

        let call_site = PulleyABICallSite::from_ptr(
            self.lower_ctx.sigs(),
            callee_sig,
            callee,
            IsTailCall::Yes,
            caller_conv,
            self.backend.flags().clone(),
        );
        call_site.emit_return_call(self.lower_ctx, args);

        InstOutput::new()
    }

    fn lower_br_table(&mut self, _index: Reg, _targets: &[MachLabel]) -> Unit {
        todo!()
    }

    fn vreg_new(&mut self, r: Reg) -> VReg {
        VReg::new(r).unwrap()
    }
    fn writable_vreg_new(&mut self, r: WritableReg) -> WritableVReg {
        r.map(|wr| VReg::new(wr).unwrap())
    }
    fn writable_vreg_to_vreg(&mut self, arg0: WritableVReg) -> VReg {
        arg0.to_reg()
    }
    fn writable_vreg_to_writable_reg(&mut self, arg0: WritableVReg) -> WritableReg {
        arg0.map(|vr| vr.to_reg())
    }
    fn vreg_to_reg(&mut self, arg0: VReg) -> Reg {
        *arg0
    }
    fn xreg_new(&mut self, r: Reg) -> XReg {
        XReg::new(r).unwrap()
    }
    fn writable_xreg_new(&mut self, r: WritableReg) -> WritableXReg {
        r.map(|wr| XReg::new(wr).unwrap())
    }
    fn writable_xreg_to_xreg(&mut self, arg0: WritableXReg) -> XReg {
        arg0.to_reg()
    }
    fn writable_xreg_to_writable_reg(&mut self, arg0: WritableXReg) -> WritableReg {
        arg0.map(|xr| xr.to_reg())
    }
    fn xreg_to_reg(&mut self, arg0: XReg) -> Reg {
        *arg0
    }
    fn freg_new(&mut self, r: Reg) -> FReg {
        FReg::new(r).unwrap()
    }
    fn writable_freg_new(&mut self, r: WritableReg) -> WritableFReg {
        r.map(|wr| FReg::new(wr).unwrap())
    }
    fn writable_freg_to_freg(&mut self, arg0: WritableFReg) -> FReg {
        arg0.to_reg()
    }
    fn writable_freg_to_writable_reg(&mut self, arg0: WritableFReg) -> WritableReg {
        arg0.map(|fr| fr.to_reg())
    }
    fn freg_to_reg(&mut self, arg0: FReg) -> Reg {
        *arg0
    }

    #[inline]
    fn emit(&mut self, arg0: &MInst) -> Unit {
        self.lower_ctx.emit(arg0.clone().into());
    }
}

/// The main entry point for lowering with ISLE.
pub(crate) fn lower<P>(
    lower_ctx: &mut Lower<InstAndKind<P>>,
    backend: &PulleyBackend<P>,
    inst: Inst,
) -> Option<InstOutput>
where
    P: PulleyTargetKind,
{
    // TODO: reuse the ISLE context across lowerings so we can reuse its
    // internal heap allocations.
    let mut isle_ctx = PulleyIsleContext::new(lower_ctx, backend);
    generated_code::constructor_lower(&mut isle_ctx, inst)
}

/// The main entry point for branch lowering with ISLE.
pub(crate) fn lower_branch<P>(
    lower_ctx: &mut Lower<InstAndKind<P>>,
    backend: &PulleyBackend<P>,
    branch: Inst,
    targets: &[MachLabel],
) -> Option<()>
where
    P: PulleyTargetKind,
{
    // TODO: reuse the ISLE context across lowerings so we can reuse its
    // internal heap allocations.
    let mut isle_ctx = PulleyIsleContext::new(lower_ctx, backend);
    generated_code::constructor_lower_branch(&mut isle_ctx, branch, targets)
}
