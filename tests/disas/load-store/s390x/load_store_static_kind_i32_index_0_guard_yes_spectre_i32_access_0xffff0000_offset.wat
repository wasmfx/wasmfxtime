;;! target = "s390x"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -O static-memory-forced -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i32 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store offset=0xffff0000)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load offset=0xffff0000))

;; function u0:0:
;;   unwind DefineNewFrame { offset_upward_to_caller_sp: 160, offset_downward_to_clobbers: 0 }
;;   stmg %r7, %r15, 56(%r15)
;;   unwind SaveReg { clobber_offset: 56, reg: p7i }
;;   unwind SaveReg { clobber_offset: 64, reg: p8i }
;;   unwind SaveReg { clobber_offset: 72, reg: p9i }
;;   unwind SaveReg { clobber_offset: 80, reg: p10i }
;;   unwind SaveReg { clobber_offset: 88, reg: p11i }
;;   unwind SaveReg { clobber_offset: 96, reg: p12i }
;;   unwind SaveReg { clobber_offset: 104, reg: p13i }
;;   unwind SaveReg { clobber_offset: 112, reg: p14i }
;;   unwind SaveReg { clobber_offset: 120, reg: p15i }
;;   unwind StackAlloc { size: 0 }
;; block0:
;;   lgr %r3, %r2
;;   llgfr %r2, %r4
;;   lghi %r4, 0
;;   lgr %r10, %r3
;;   lgr %r3, %r2
;;   ag %r3, 80(%r10)
;;   llilh %r7, 65535
;;   agr %r3, %r7
;;   clgfi %r2, 65532
;;   locgrh %r3, %r4
;;   strv %r5, 0(%r3)
;;   jg label1
;; block1:
;;   lmg %r7, %r15, 56(%r15)
;;   br %r14
;;
;; function u0:1:
;;   unwind DefineNewFrame { offset_upward_to_caller_sp: 160, offset_downward_to_clobbers: 0 }
;;   unwind StackAlloc { size: 0 }
;; block0:
;;   llgfr %r5, %r4
;;   lghi %r4, 0
;;   lgr %r3, %r5
;;   ag %r3, 80(%r2)
;;   llilh %r2, 65535
;;   agr %r3, %r2
;;   clgfi %r5, 65532
;;   locgrh %r3, %r4
;;   lrv %r2, 0(%r3)
;;   jg label1
;; block1:
;;   br %r14
