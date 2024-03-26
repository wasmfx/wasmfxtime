;;! target = "s390x"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -W memory64 -O static-memory-maximum-size=0 -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

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
;;   1a: aghi    %r3, -4
;;   1e: clgr    %r4, %r3
;;   22: jgh     0x3c
;;   28: lg      %r3, 0x50(%r2)
;;   2e: strv    %r5, 0(%r4, %r3)
;;   34: lmg     %r14, %r15, 0x110(%r15)
;;   3a: br      %r14
;;   3c: .byte   0x00, 0x00
;;
;; wasm[0]::function[1]:
;;   40: stmg    %r14, %r15, 0x70(%r15)
;;   46: lgr     %r1, %r15
;;   4a: aghi    %r15, -0xa0
;;   4e: stg     %r1, 0(%r15)
;;   54: lg      %r5, 0x58(%r2)
;;   5a: aghi    %r5, -4
;;   5e: clgr    %r4, %r5
;;   62: jgh     0x7c
;;   68: lg      %r3, 0x50(%r2)
;;   6e: lrv     %r2, 0(%r4, %r3)
;;   74: lmg     %r14, %r15, 0x110(%r15)
;;   7a: br      %r14
;;   7c: .byte   0x00, 0x00
