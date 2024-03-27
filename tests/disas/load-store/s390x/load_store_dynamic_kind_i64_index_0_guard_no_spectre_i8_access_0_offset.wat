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
    i32.store8 offset=0)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load8_u offset=0))

;; wasm[0]::function[0]:
;;    0: stmg    %r14, %r15, 0x70(%r15)
;;    6: lgr     %r1, %r15
;;    a: aghi    %r15, -0xa0
;;    e: stg     %r1, 0(%r15)
;;   14: lg      %r3, 0x58(%r2)
;;   1a: clgr    %r4, %r3
;;   1e: jghe    0x36
;;   24: lg      %r2, 0x50(%r2)
;;   2a: stc     %r5, 0(%r4, %r2)
;;   2e: lmg     %r14, %r15, 0x110(%r15)
;;   34: br      %r14
;;   36: .byte   0x00, 0x00
;;
;; wasm[0]::function[1]:
;;   38: stmg    %r14, %r15, 0x70(%r15)
;;   3e: lgr     %r1, %r15
;;   42: aghi    %r15, -0xa0
;;   46: stg     %r1, 0(%r15)
;;   4c: lg      %r5, 0x58(%r2)
;;   52: clgr    %r4, %r5
;;   56: jghe    0x70
;;   5c: lg      %r2, 0x50(%r2)
;;   62: llc     %r2, 0(%r4, %r2)
;;   68: lmg     %r14, %r15, 0x110(%r15)
;;   6e: br      %r14
;;   70: .byte   0x00, 0x00
