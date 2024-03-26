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
    i32.store offset=0)

  (func (export "do_load") (param i32) (result i32)
    local.get 0
    i32.load offset=0))

;; wasm[0]::function[0]:
;;    0: stmg    %r6, %r15, 0x30(%r15)
;;    6: lgr     %r1, %r15
;;    a: aghi    %r15, -0xa0
;;    e: stg     %r1, 0(%r15)
;;   14: lgr     %r6, %r2
;;   18: llgfr   %r3, %r4
;;   1c: lghi    %r2, 0
;;   20: lgr     %r8, %r6
;;   24: lgr     %r4, %r3
;;   28: ag      %r4, 0x50(%r8)
;;   2e: clgfi   %r3, 0xfffffffc
;;   34: locgrh  %r4, %r2
;;   38: strv    %r5, 0(%r4)
;;   3e: lmg     %r6, %r15, 0xd0(%r15)
;;   44: br      %r14
;;
;; wasm[0]::function[1]:
;;   48: stmg    %r14, %r15, 0x70(%r15)
;;   4e: lgr     %r1, %r15
;;   52: aghi    %r15, -0xa0
;;   56: stg     %r1, 0(%r15)
;;   5c: lgr     %r5, %r2
;;   60: llgfr   %r3, %r4
;;   64: lghi    %r2, 0
;;   68: lgr     %r4, %r5
;;   6c: lgr     %r5, %r3
;;   70: ag      %r5, 0x50(%r4)
;;   76: clgfi   %r3, 0xfffffffc
;;   7c: locgrh  %r5, %r2
;;   80: lrv     %r2, 0(%r5)
;;   86: lmg     %r14, %r15, 0x110(%r15)
;;   8c: br      %r14
