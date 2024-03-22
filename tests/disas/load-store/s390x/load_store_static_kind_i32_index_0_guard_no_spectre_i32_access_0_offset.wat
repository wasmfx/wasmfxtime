;;! target = "s390x"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -O static-memory-forced -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i32 1)

  (func (export "do_store") (param i32 i32)
    local.get 0
    local.get 1
    i32.store offset=0)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load offset=0))

;; function u0:0:
;;   unwind DefineNewFrame { offset_upward_to_caller_sp: 160, offset_downward_to_clobbers: 0 }
;;   unwind StackAlloc { size: 0 }
;; block0:
;;   llgfr %r4, %r4
;;   clgfi %r4, 4294967292
;;   jgh label3 ; jg label1
;; block1:
;;   lg %r2, 80(%r2)
;;   strv %r5, 0(%r4,%r2)
;;   jg label2
;; block2:
;;   br %r14
;; block3:
;;   .word 0x0000 # trap=heap_oob
;;
;; function u0:1:
;;   unwind DefineNewFrame { offset_upward_to_caller_sp: 160, offset_downward_to_clobbers: 0 }
;;   unwind StackAlloc { size: 0 }
;; block0:
;;   llgfr %r4, %r4
;;   clgfi %r4, 4294967292
;;   jgh label3 ; jg label1
;; block1:
;;   lg %r2, 80(%r2)
;;   lrv %r2, 0(%r4,%r2)
;;   jg label2
;; block2:
;;   br %r14
;; block3:
;;   .word 0x0000 # trap=heap_oob
