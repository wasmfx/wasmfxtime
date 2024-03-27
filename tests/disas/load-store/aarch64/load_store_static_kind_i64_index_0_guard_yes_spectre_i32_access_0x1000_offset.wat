;;! target = "aarch64"
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
;;    0: stp     x29, x30, [sp, #-0x10]!
;;    4: mov     x29, sp
;;    8: mov     x10, #0
;;    c: ldr     x11, [x0, #0x50]
;;   10: add     x11, x11, x2
;;   14: add     x11, x11, #1, lsl #12
;;   18: mov     w9, #-0x1004
;;   1c: cmp     x2, x9
;;   20: csel    x12, x10, x11, hi
;;   24: csdb
;;   28: str     w3, [x12]
;;   2c: ldp     x29, x30, [sp], #0x10
;;   30: ret
;;
;; wasm[0]::function[1]:
;;   40: stp     x29, x30, [sp, #-0x10]!
;;   44: mov     x29, sp
;;   48: mov     x10, #0
;;   4c: ldr     x11, [x0, #0x50]
;;   50: add     x11, x11, x2
;;   54: add     x11, x11, #1, lsl #12
;;   58: mov     w9, #-0x1004
;;   5c: cmp     x2, x9
;;   60: csel    x12, x10, x11, hi
;;   64: csdb
;;   68: ldr     w0, [x12]
;;   6c: ldp     x29, x30, [sp], #0x10
;;   70: ret
