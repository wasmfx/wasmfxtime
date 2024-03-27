;;! target = "s390x"
;;! test = "compile"
;;! flags = " -C cranelift-enable-heap-access-spectre-mitigation -W memory64 -O static-memory-forced -O static-memory-guard-size=0 -O dynamic-memory-guard-size=0"

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i64 1)

  (func (export "do_store") (param i64 i32)
    local.get 0
    local.get 1
    i32.store offset=0x1000)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load offset=0x1000))

;; wasm[0]::function[0]:
;;    0: stmg    %r14, %r15, 0x70(%r15)
;;    6: lgr     %r1, %r15
;;    a: aghi    %r15, -0xa0
;;    e: stg     %r1, 0(%r15)
;;   14: lghi    %r14, 0
;;   18: lgr     %r3, %r4
;;   1c: ag      %r3, 0x50(%r2)
;;   22: aghi    %r3, 0x1000
;;   26: clgfi   %r4, 0xffffeffc
;;   2c: locgrh  %r3, %r14
;;   30: strv    %r5, 0(%r3)
;;   36: lmg     %r14, %r15, 0x110(%r15)
;;   3c: br      %r14
;;
;; wasm[0]::function[1]:
;;   40: stmg    %r14, %r15, 0x70(%r15)
;;   46: lgr     %r1, %r15
;;   4a: aghi    %r15, -0xa0
;;   4e: stg     %r1, 0(%r15)
;;   54: lghi    %r3, 0
;;   58: lgr     %r5, %r4
;;   5c: ag      %r5, 0x50(%r2)
;;   62: aghi    %r5, 0x1000
;;   66: clgfi   %r4, 0xffffeffc
;;   6c: locgrh  %r5, %r3
;;   70: lrv     %r2, 0(%r5)
;;   76: lmg     %r14, %r15, 0x110(%r15)
;;   7c: br      %r14
