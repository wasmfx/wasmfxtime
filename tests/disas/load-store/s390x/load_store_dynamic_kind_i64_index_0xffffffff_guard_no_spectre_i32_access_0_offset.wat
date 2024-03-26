;;! target = "s390x"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -W memory64 -O static-memory-maximum-size=0 -O static-memory-guard-size=4294967295 -O dynamic-memory-guard-size=4294967295"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i64 1)

  (func (export "do_store") (param i64 i32)
    local.get 0
    local.get 1
    i32.store offset=0)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load offset=0))

;; wasm[0]::function[0]:
;;    0: stmg    %r14, %r15, 0x70(%r15)
;;    6: lgr     %r1, %r15
;;    a: aghi    %r15, -0xa0
;;    e: stg     %r1, 0(%r15)
;;   14: lg      %r3, 0x58(%r2)
;;   1a: clgr    %r4, %r3
;;   1e: jgh     0x38
;;   24: lg      %r2, 0x50(%r2)
;;   2a: strv    %r5, 0(%r4, %r2)
;;   30: lmg     %r14, %r15, 0x110(%r15)
;;   36: br      %r14
;;   38: .byte   0x00, 0x00
;;
;; wasm[0]::function[1]:
;;   3c: stmg    %r14, %r15, 0x70(%r15)
;;   42: lgr     %r1, %r15
;;   46: aghi    %r15, -0xa0
;;   4a: stg     %r1, 0(%r15)
;;   50: lg      %r5, 0x58(%r2)
;;   56: clgr    %r4, %r5
;;   5a: jgh     0x74
;;   60: lg      %r2, 0x50(%r2)
;;   66: lrv     %r2, 0(%r4, %r2)
;;   6c: lmg     %r14, %r15, 0x110(%r15)
;;   72: br      %r14
;;   74: .byte   0x00, 0x00
