;;! target = "s390x"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation=false -O static-memory-maximum-size=0 -O static-memory-guard-size=4294967295 -O dynamic-memory-guard-size=4294967295"

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

;; wasm[0]::function[0]:
;;    0: stmg    %r14, %r15, 0x70(%r15)
;;    6: lgr     %r1, %r15
;;    a: aghi    %r15, -0xa0
;;    e: stg     %r1, 0(%r15)
;;   14: lgr     %r3, %r4
;;   18: lg      %r4, 0x58(%r2)
;;   1e: llgfr   %r3, %r3
;;   22: clgr    %r3, %r4
;;   26: jgh     0x44
;;   2c: ag      %r3, 0x50(%r2)
;;   32: llilh   %r4, 0xffff
;;   36: strv    %r5, 0(%r4, %r3)
;;   3c: lmg     %r14, %r15, 0x110(%r15)
;;   42: br      %r14
;;   44: .byte   0x00, 0x00
;;
;; wasm[0]::function[1]:
;;   48: stmg    %r14, %r15, 0x70(%r15)
;;   4e: lgr     %r1, %r15
;;   52: aghi    %r15, -0xa0
;;   56: stg     %r1, 0(%r15)
;;   5c: lg      %r3, 0x58(%r2)
;;   62: llgfr   %r5, %r4
;;   66: clgr    %r5, %r3
;;   6a: jgh     0x88
;;   70: ag      %r5, 0x50(%r2)
;;   76: llilh   %r4, 0xffff
;;   7a: lrv     %r2, 0(%r4, %r5)
;;   80: lmg     %r14, %r15, 0x110(%r15)
;;   86: br      %r14
;;   88: .byte   0x00, 0x00
